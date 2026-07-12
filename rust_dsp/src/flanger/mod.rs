// MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW

use wasm_bindgen::prelude::*;

use crate::{
    SamplePair,
    buffer::DspBuffer,
    delay_line::DelayLine,
    lerp,
    util::{self, Interpolator, Zippable},
};

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct FlangerInstanceParams {
    pub delay: f32,
    pub panning: f32,
    pub mix: f32,
    pub feedmix: f32,
    pub voices: f32,
}
impl FlangerInstanceParams {
    fn split(&self, sample_rate: f32) -> (FlangerParams, FlangerParams) {
        let delay = self.delay * 0.000024414063 * sample_rate;
        let panning = self.panning * 0.5 + 0.5;
        let mix = self.mix * (1.0 / 63.0);
        let feedmix = self.feedmix * (1.0 / 64.0);
        let voices = self.voices;
        (
            FlangerParams {
                delay: delay * (1.0 - panning),
                mix,
                feedmix,
                voices,
            },
            FlangerParams {
                delay: delay * panning,
                mix,
                feedmix,
                voices,
            },
        )
    }
}

#[wasm_bindgen]
#[derive(Default)]
pub struct FlangerInstance {
    pub start: FlangerInstanceParams,
    pub end: FlangerInstanceParams,

    shifter_l: Flanger,
    shifter_r: Flanger,
}
#[wasm_bindgen]
impl FlangerInstance {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Default::default()
    }
    #[wasm_bindgen]
    pub fn process(&mut self, buffer: &mut DspBuffer) {
        let Self { start, end, .. } = self;

        let run_length = buffer.run_length as f32;
        let sample_rate = buffer.sample_rate;

        let (start_l, start_r) = start.split(sample_rate);
        let (end_l, end_r) = end.split(sample_rate);

        let (left, right) = buffer.as_channels();

        self.shifter_l
            .process(left, util::interpolate(run_length, start_l, end_l));
        self.shifter_r
            .process(right, util::interpolate(run_length, start_r, end_r));
    }
}

struct FlangerParams {
    delay: f32,
    mix: f32,
    feedmix: f32,
    voices: f32,
}
impl Zippable for FlangerParams {
    fn zip(&self, other: &Self, f: impl Fn(f32, f32) -> f32) -> Self {
        Self {
            delay: f(self.delay, other.delay),
            mix: f(self.mix, other.mix),
            feedmix: f(self.feedmix, other.feedmix),
            voices: f(self.voices, other.voices),
        }
    }
}

/// mono flanger using linear interpolation
struct Flanger {
    // not actually a SamplePair. left side is delayed input, right side is delayed output.
    delay_line: DelayLine<SamplePair>,
}
impl Default for Flanger {
    fn default() -> Self {
        Self {
            // 200ms at 48kHz
            delay_line: DelayLine::new(200 * (48000 / 1000) + 42),
        }
    }
}
impl Flanger {
    fn process(&mut self, buf: &mut [f32], mut interpolator: Interpolator<FlangerParams>) {
        for sample in buf {
            let params = interpolator.next();

            let (dx, dy) = if (params.voices - 1.0).abs() < 1e-5 {
                let SamplePair { l, r } = self.delay_line.compute(params.delay);
                (l, r)
            } else {
                let mut result = 0.0;
                let num_voices_int = params.voices.ceil() as usize;
                let inv_num_voices = params.voices.recip().min(1.0);
                for i in 1..=num_voices_int {
                    let SamplePair { l: mut val, .. } = self
                        .delay_line
                        .compute(params.delay * inv_num_voices * i as f32);
                    if i as f32 - params.voices > 0.0 {
                        val *= 1.0 - (i as f32 - params.voices);
                    }
                    result += val;
                }
                let SamplePair { r, .. } = self.delay_line.compute(params.delay);

                (result, r)
            };

            let x = *sample;
            let y = lerp(x, lerp(dx, dy, params.feedmix), params.mix);
            self.delay_line.push(SamplePair { l: x, r: y });
            *sample = y;
        }
    }
}
