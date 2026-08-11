use core::simd::{f32x4, simd_swizzle};
use std::iter::zip;

use wasm_bindgen::prelude::*;

use crate::util::{self, Interpolator};

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
struct PhaserInstanceParams {
    pub mix: f32,
    pub freq: f32,
    pub feedback: f32,
}
#[wasm_bindgen]
impl PhaserInstanceParams {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Default::default()
    }
}

#[wasm_bindgen]
#[derive(Default)]
struct PhaserInstance {
    pub frame_size: usize,
    pub disperse: bool,
    legacy_behavior: bool,

    i_break_coef: Interpolator<f32>,
    i_feedback_mult: Interpolator<f32>,
    i_mix: Interpolator<f32>,

    imp: SimdMonoPhaser,
    prev_simd_output: f32x4,
    cur_simd: f32x4,
    simd_index: u8,
}

fn get_break_coef(freq: f32, sample_rate: f32) -> f32 {
    let break_t = f32::tan(0.5 * crate::filters::to_w0(freq, sample_rate));
    (break_t - 1.0) / (break_t + 1.0)
}

#[wasm_bindgen]
impl PhaserInstance {
    #[wasm_bindgen(constructor)]
    pub fn new(frame_size: usize) -> Self {
        Self {
            frame_size,
            ..Default::default()
        }
    }

    #[wasm_bindgen(setter)]
    pub fn set_num_stages(&mut self, num_stages: i32) {
        self.imp.resize(num_stages.try_into().unwrap_or(0));
    }
    #[wasm_bindgen(setter)]
    pub fn set_legacy_behavior(&mut self, legacy_behavior: bool) {
        if self.legacy_behavior == legacy_behavior {
            return;
        }
        self.legacy_behavior = legacy_behavior;

        if self.legacy_behavior {
            self.prev_simd_output = Default::default();
            self.cur_simd = Default::default();
            self.simd_index = 0;
        }
    }

    #[wasm_bindgen]
    pub fn begin(
        &mut self,
        start: PhaserInstanceParams,
        end: PhaserInstanceParams,
        sample_rate: f32,
        run_length: f32,
    ) {
        self.i_break_coef = util::interpolate(
            run_length,
            get_break_coef(start.freq, sample_rate),
            get_break_coef(end.freq, sample_rate),
        );

        self.i_feedback_mult = util::interpolate(run_length, start.feedback, end.feedback);

        self.i_mix = util::interpolate(run_length, start.mix, end.mix);
    }

    #[wasm_bindgen]
    pub fn process(&mut self, sample: f32) -> f32 {
        let Self {
            prev_simd_output,
            cur_simd,

            simd_index,
            ..
        } = self;

        if self.legacy_behavior {
            let prev_output = &mut prev_simd_output.as_mut_array()[0];
            let mut propagated = sample;
            propagated += *prev_output * self.i_feedback_mult.next();
            self.imp
                .compute_direct(self.i_break_coef.next(), &mut propagated);
            *prev_output = propagated;

            if self.disperse {
                sample + (propagated - sample) * self.i_mix.next()
            } else {
                sample + propagated * self.i_mix.next()
            }
        } else {
            assert!(*simd_index < 4);
            let output_val =
                std::mem::replace(&mut cur_simd.as_mut_array()[*simd_index as usize], sample);
            *simd_index += 1;
            if *simd_index >= 4 {
                *simd_index = 0;
                let sample_simd = *cur_simd;
                let mut propagated = sample_simd;

                propagated += self.prev_simd_output * f32x4::splat(self.i_feedback_mult.next());
                self.imp
                    .compute(self.i_break_coef.next_simd(), &mut propagated);

                self.prev_simd_output = propagated;

                *cur_simd = if self.disperse {
                    sample_simd + (propagated - sample_simd) * self.i_mix.next_simd()
                } else {
                    sample_simd + propagated * self.i_mix.next_simd()
                };
            }

            output_val
        }
    }
}

/// SIMD-accelerated one-channel phaser.
#[derive(Default)]
struct SimdMonoPhaser {
    prev_inputs: Vec<f32x4>,
    outputs: Vec<f32x4>,
    // invariant: `self.simd_len() * 4 >= self.num_stages`
    num_stages: usize,
}

impl SimdMonoPhaser {
    fn resize(&mut self, num_stages: usize) {
        if self.num_stages == num_stages {
            return;
        }
        let vecsize = num_stages.div_ceil(f32x4::LEN);
        self.prev_inputs.resize(vecsize, f32x4::splat(0.0));
        self.outputs.resize(vecsize, f32x4::splat(0.0));
        self.num_stages = num_stages;
        for unused in self.num_stages..vecsize * f32x4::LEN {
            self.get_single_mut(unused).reset();
        }
    }

    fn get_simd(&self, index: usize) -> SimdMonoPhaserStage {
        SimdMonoPhaserStage {
            prev_input: self.prev_inputs[index],
            output: self.outputs[index],
        }
    }
    fn set_simd(&mut self, index: usize, stage: SimdMonoPhaserStage) {
        self.prev_inputs[index] = stage.prev_input;
        self.outputs[index] = stage.output;
    }

    fn get_single_mut(&mut self, index: usize) -> MonoPhaserStageMut<'_> {
        let prev_inputs: &mut [f32] = bytemuck::must_cast_slice_mut(&mut self.prev_inputs);
        let outputs: &mut [f32] = bytemuck::must_cast_slice_mut(&mut self.outputs);
        MonoPhaserStageMut {
            prev_input: unsafe { prev_inputs.get_unchecked_mut(index) },
            output: unsafe { outputs.get_unchecked_mut(index) },
        }
    }

    pub fn compute_direct(&mut self, break_coef: f32, val: &mut f32) {
        for stage in 0..self.num_stages {
            self.get_single_mut(stage).compute(break_coef, val);
        }
    }

    pub fn compute(&mut self, break_coef_simd: f32x4, val_simd: &mut f32x4) {
        let simd_len = self.num_stages / 4;
        assert_eq!(self.outputs.len(), self.prev_inputs.len());
        assert!(simd_len <= self.outputs.len());

        // arbitrary threshold below which simd isn't worth it
        if simd_len < 4 {
            for (val, break_coef) in zip(val_simd.as_mut_array(), break_coef_simd.to_array()) {
                self.compute_direct(break_coef, val);
            }
            return;
        }

        // SOME people *cough cough* use hundreds of phaser stages on their instruments. i respect that but at that point might as well do computations in parallel to save cpu time. right?
        // anyways, this algorithm does that for a set of 4 mono samples. the gist is:
        // consider a sample being processed through some prefix of the phaser stages. in order for it to exist:
        // - the previous sample needs to be processed by the current phaser
        // - the previous phaser needs to have processed the current sample
        // so no simple "do it in parallel" approach here. instead, the solution is to stagger each batch of 4 samples:
        //
        //    s1 s2 s3 s4
        // p1 .. .. .. XX
        // p2 .. .. XX |
        // p3 .. XX |  V
        // p4 XX |  V
        // p5 |  V
        // p6 V
        // ...
        //
        // where `XX` represents the values stored by the f32x4 in `val_simd` and `..` represents values that were previously processed.

        // stagger the samples to prepare for the simd hot loop
        {
            for (i, (val, break_coef)) in
                zip(val_simd.as_mut_array(), break_coef_simd.to_array()).enumerate()
            {
                for stage in 0..3 - i {
                    self.get_single_mut(stage).compute(break_coef, val);
                }
            }
        }

        // reverse the value; this aligns the simd values with the stage values
        // because e.g. the first value needs to reach the last stage before any of the other values reach it
        *val_simd = val_simd.reverse();

        let mut cur_stage = self.get_simd(0);
        for i in 1..simd_len {
            let mut next_stage = self.get_simd(i);

            for _iter in 0..4 {
                cur_stage.compute(break_coef_simd, val_simd);
                concat_rotate(&mut cur_stage.prev_input, &mut next_stage.prev_input);
                concat_rotate(&mut cur_stage.output, &mut next_stage.output);
            }

            // by rotating 4 times, next_stage is now the new value of cur_stage. set it!
            self.set_simd(i - 1, next_stage);
            // this also means that cur_stage is now the new value of next_stage; continue to the next loop iteration
        }
        self.set_simd(simd_len - 1, cur_stage);

        // put the values back where they started
        *val_simd = val_simd.reverse();

        // destagger the samples to clean up
        {
            let non_simd_start = (simd_len - 1) * f32x4::LEN;
            for (i, (val, break_coef)) in
                zip(val_simd.as_mut_array(), break_coef_simd.to_array()).enumerate()
            {
                for stage in non_simd_start + 3 - i..self.num_stages {
                    self.get_single_mut(stage).compute(break_coef, val);
                }
            }
        }
    }
}

fn concat_rotate(s1: &mut f32x4, s2: &mut f32x4) {
    (*s1, *s2) = (
        simd_swizzle!(*s1, *s2, [1, 2, 3, 4]),
        simd_swizzle!(*s1, *s2, [5, 6, 7, 0]),
    );
}

struct SimdMonoPhaserStage {
    prev_input: f32x4,
    output: f32x4,
}
impl SimdMonoPhaserStage {
    fn compute(&mut self, break_coef: f32x4, input: &mut f32x4) {
        self.output = break_coef * (*input - self.output) + self.prev_input;
        self.prev_input = *input;
        *input = self.output;
    }
}

struct MonoPhaserStageMut<'a> {
    prev_input: &'a mut f32,
    output: &'a mut f32,
}
impl MonoPhaserStageMut<'_> {
    fn compute(&mut self, break_coef: f32, val: &mut f32) {
        *self.output = break_coef * (*val - *self.output) + *self.prev_input;
        *self.prev_input = *val;
        *val = *self.output;
    }
    fn reset(&mut self) {
        *self.prev_input = 0.0;
        *self.output = 0.0;
    }
}
