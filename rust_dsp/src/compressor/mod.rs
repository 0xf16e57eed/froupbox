//! Compressor algorithm taken from cy!box, which is based on CALF's compressor.
//! https://gitlab.com/cyphers-stuff/cybox/-/blob/31c2eda59748f321a09141b41552d8e65a755dfe/dsp/beepbox/src/effect/compressor.rs

use std::f32;

use wasm_bindgen::prelude::*;

use crate::{
    SamplePair,
    buffer::DspBuffer,
    compressor::comp::{Compressor, CompressorParams},
    filters::{Crossover, CrossoverCoefficients, to_w0},
    util::{self, Interpolator},
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
#[wasm_bindgen]
impl CompressorInstanceParams {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Default::default()
    }
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

#[derive(Default)]
#[wasm_bindgen]
struct CompressorInstance {
    split_lo_mid: Crossover,
    split_mid_hi: Crossover,

    lo: Compressor,
    mid: Compressor,
    hi: Compressor,

    coef_lo_mid: CrossoverCoefficients,
    coef_mid_hi: CrossoverCoefficients,
    comp_params: Interpolator<CompressorParams>,

    lo_mult: Interpolator<f32>,
    mid_mult: Interpolator<f32>,
    hi_mult: Interpolator<f32>,
}

#[wasm_bindgen]
impl CompressorInstance {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Default::default()
    }

    #[wasm_bindgen]
    pub fn begin(
        &mut self,
        start: CompressorInstanceParams,
        end: CompressorInstanceParams,
        sample_rate: f32,
        run_length: f32,
    ) {
        self.coef_lo_mid = CrossoverCoefficients::new(to_w0(start.freq_lo_mid, sample_rate));
        self.coef_mid_hi = CrossoverCoefficients::new(to_w0(start.freq_mid_hi, sample_rate));
        self.comp_params = util::interpolate(
            run_length,
            start.comp_params(sample_rate),
            end.comp_params(sample_rate),
        );

        self.lo_mult = util::interpolate(run_length, start.lo_gain, end.lo_gain);
        self.mid_mult = util::interpolate(run_length, start.mid_gain, end.mid_gain);
        self.hi_mult = util::interpolate(run_length, start.hi_gain, end.hi_gain);
    }

    #[wasm_bindgen]
    pub fn process(&mut self, buffer: &mut DspBuffer) {
        for (l, r) in buffer.as_zipped() {
            let [mut lo, mut mid, mut hi] = [SamplePair { l: *l, r: *r }; 3];

            self.split_mid_hi.run(&self.coef_mid_hi, &mut mid, &mut hi);
            self.split_lo_mid.run(&self.coef_lo_mid, &mut lo, &mut mid);

            let cur_comp_params = self.comp_params.next();

            let sample = self.lo.process(&cur_comp_params, lo) * self.lo_mult.next()
                + self.mid.process(&cur_comp_params, mid) * self.mid_mult.next()
                + self.hi.process(&cur_comp_params, hi) * self.hi_mult.next();

            *l = sample.l.clamp(-1.0, 1.0);
            *r = sample.r.clamp(-1.0, 1.0);
        }
    }
}
