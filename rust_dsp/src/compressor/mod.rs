//! Compressor algorithm taken from cy!box, which is based on CALF's compressor.
//! https://gitlab.com/cyphers-stuff/cybox/-/blob/31c2eda59748f321a09141b41552d8e65a755dfe/dsp/beepbox/src/effect/compressor.rs

use std::f32;

use wasm_bindgen::prelude::*;

use crate::{
    SamplePair,
    buffer::DspBuffer,
    compressor::comp::{Compressor, CompressorParams},
    filters::{Crossover, CrossoverCoefficients, to_w0},
    util,
};
mod comp;

#[wasm_bindgen]
#[derive(Default, Clone, Copy, Debug)]
struct CompressorInstanceParams {
    pub attack: f32,
    pub decay: f32,
    pub threshold: f32,

    pub ratio_up: f32,
    pub ratio_down: f32,

    pub freq_lo_mid: f32,
    pub freq_mid_hi: f32,

    pub lo_gain: f32,
    pub mid_gain: f32,
    pub hi_gain: f32,
}
impl CompressorInstanceParams {
    fn comp_params(&self, sample_rate: f32) -> CompressorParams {
        let mut params = CompressorParams::new(sample_rate);
        params.attack = self.attack;
        params.decay = self.decay;
        params.threshold = self.threshold;
        params.ratio_up = self.ratio_up;
        params.ratio_down = self.ratio_down;
        params
    }
}

#[wasm_bindgen]
struct CompressorInstance {
    pub start: CompressorInstanceParams,
    pub end: CompressorInstanceParams,

    split_lo_mid: Crossover,
    split_mid_hi: Crossover,

    lo: Compressor,
    mid: Compressor,
    hi: Compressor,
}

#[wasm_bindgen]
impl CompressorInstance {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            split_lo_mid: Default::default(),
            split_mid_hi: Default::default(),

            lo: Default::default(),
            mid: Default::default(),
            hi: Default::default(),

            end: Default::default(),
            start: Default::default(),
        }
    }

    #[wasm_bindgen]
    pub fn process(&mut self, buffer: &mut DspBuffer) {
        let Self {
            ref start, ref end, ..
        } = *self;

        let run_length_f32 = buffer.run_length as f32;
        let sample_rate = buffer.sample_rate;

        if start.freq_lo_mid < 10.0 {
            // compressor hasn't been initialized yet; ignore
            return;
        }

        let coef_lo_mid = CrossoverCoefficients::new(to_w0(start.freq_lo_mid, sample_rate));
        let coef_mid_hi = CrossoverCoefficients::new(to_w0(start.freq_mid_hi, sample_rate));
        let mut comp_params = util::interpolate(
            run_length_f32,
            start.comp_params(sample_rate),
            end.comp_params(sample_rate),
        );

        let mut lo_mult = util::interpolate(run_length_f32, start.lo_gain, end.lo_gain);
        let mut mid_mult = util::interpolate(run_length_f32, start.mid_gain, end.mid_gain);
        let mut hi_mult = util::interpolate(run_length_f32, start.hi_gain, end.hi_gain);

        for (l, r) in buffer.as_zipped() {
            let [mut lo, mut mid, mut hi] = [SamplePair { l: *l, r: *r }; 3];

            self.split_mid_hi.run(&coef_mid_hi, &mut mid, &mut hi);
            self.split_lo_mid.run(&coef_lo_mid, &mut lo, &mut mid);

            let cur_comp_params = comp_params.next();

            let sample = self.lo.process(&cur_comp_params, lo) * lo_mult.next()
                + self.mid.process(&cur_comp_params, mid) * mid_mult.next()
                + self.hi.process(&cur_comp_params, hi) * hi_mult.next();

            *l = sample.l.clamp(-1.0, 1.0);
            *r = sample.r.clamp(-1.0, 1.0);
        }
    }
}
