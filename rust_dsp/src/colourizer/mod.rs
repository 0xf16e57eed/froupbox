//! https://en.wiktionary.org/wiki/colorize#English
//! > "colourize (Canada, Oxford British English)"

use std::iter::zip;

use crate::{
    SamplePair,
    buffer::DspBuffer,
    flanger::{Flanger, FlangerParams},
    util::{self, Interpolator, Zippable},
};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Default, Clone, Copy)]
pub struct ColourizerInstanceParams {
    pub mix: f32,
    pub voices: f32,
}
#[wasm_bindgen]
impl ColourizerInstanceParams {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Default::default()
    }
}
impl ColourizerInstanceParams {
    fn mix(&self) -> f32 {
        self.mix * (1.0 / 63.0)
    }
    fn for_freq(&self, sample_rate: f32, freq: f32) -> FlangerParams {
        FlangerParams {
            delay: sample_rate / freq,
            mix: self.mix(),
            feedmix: 0.0,
            voices: self.voices,
        }
    }
}
impl Zippable for ColourizerInstanceParams {
    fn zip(&self, other: &Self, f: impl Fn(f32, f32) -> f32) -> Self {
        Self {
            mix: f(self.mix, other.mix),
            voices: f(self.voices, other.voices),
        }
    }
}

#[wasm_bindgen]
#[derive(Default)]
pub struct ColourizerInstance {
    freqs: Vec<f32>,
    prev_freqs: Vec<f32>,
    flangers: Vec<Flanger<SamplePair>>,

    output_buf: DspBuffer,
    mix_interp: Interpolator<f32>,
}

const FREQ_MIN: f32 = 20.0;

#[wasm_bindgen]
impl ColourizerInstance {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Default::default()
    }

    #[wasm_bindgen(setter)]
    pub fn set_freqs(&mut self, freqs: Vec<f32>) {
        self.prev_freqs.resize(freqs.len(), f32::NAN);
        self.freqs = freqs;

        // don't shrink self.flangers; those are expensive to create and thus keep them in the pool
        if self.freqs.len() > self.flangers.len() {
            self.flangers
                .resize_with(self.freqs.len(), Default::default);
        }
    }

    #[wasm_bindgen]
    pub fn begin(
        &mut self,
        start: ColourizerInstanceParams,
        end: ColourizerInstanceParams,
        sample_rate: f32,
        run_length: f32,
    ) {
        for ((prev_freq, &target_freq), flanger) in
            zip(zip(&mut self.prev_freqs, &self.freqs), &mut self.flangers)
        {
            if target_freq < FREQ_MIN {
                // freq either -1 (nonexistent) or too low
                *prev_freq = f32::NAN;
                continue;
            }
            // TODO: i tried interpolating between prev_freq and target_freq but it sounds pretty terrible
            // find a better way sometime?
            // if prev_freq.is_nan() {
            //     *prev_freq = target_freq;
            // }
            let params_start = start.for_freq(sample_rate, target_freq);
            let params_end = end.for_freq(sample_rate, target_freq);

            let max_delay_samples = params_start.total_delay().max(params_end.total_delay());
            flanger.delay_line.reserve_at_least(max_delay_samples);

            flanger.interpolator = util::interpolate(run_length, params_start, params_end);
        }

        self.mix_interp = util::interpolate(run_length, start.mix(), end.mix());
    }

    #[wasm_bindgen]
    pub fn process(&mut self, buffer: &mut DspBuffer) {
        if self.output_buf.frame_size() != buffer.frame_size() {
            self.output_buf = DspBuffer::new(buffer.frame_size());
        }
        self.output_buf.run_length = buffer.run_length;

        if (self.mix_interp.val - 1.0).abs() < 1e-3 && self.mix_interp.diff.abs() <= 1e-3 {
            self.output_buf.clear();
        } else {
            for ((input_l, input_r), (output_l, output_r)) in
                zip(buffer.as_zipped(), self.output_buf.as_zipped())
            {
                let dry = 1.0 - self.mix_interp.next();
                *output_l = *input_l * dry;
                *output_r = *input_r * dry;
            }
        }

        for (&freq, flanger) in zip(&self.freqs, &mut self.flangers) {
            if freq < FREQ_MIN {
                // freq either -1 (nonexistent) or too low
                continue;
            }
            for ((input_l, input_r), (output_l, output_r)) in
                zip(buffer.as_zipped(), self.output_buf.as_zipped())
            {
                let output = flanger.process(
                    SamplePair {
                        l: *input_l,
                        r: *input_r,
                    },
                    true,
                );
                *output_l += output.l;
                *output_r += output.r;
            }
        }

        buffer.set(&mut self.output_buf);
    }
}
