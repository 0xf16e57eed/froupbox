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
#[derive(Debug, Default, Clone, Copy)]
pub struct FlangerInstanceParams {
    pub delay: f32,
    pub panning: f32,
    pub mix: f32,
    pub feedmix: f32,
    pub voices: f32,
}
#[wasm_bindgen]
impl FlangerInstanceParams {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Default::default()
    }

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
    pub use_larger_delay_line: bool,

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
    pub fn begin(
        &mut self,
        start: FlangerInstanceParams,
        end: FlangerInstanceParams,
        sample_rate: f32,
        run_length: f32,
    ) {
        let (start_l, start_r) = start.split(sample_rate);
        let (end_l, end_r) = end.split(sample_rate);

        self.shifter_l.interpolator = util::interpolate(run_length, start_l, end_l);
        self.shifter_r.interpolator = util::interpolate(run_length, start_r, end_r);

        let delay_line_size = if self.use_larger_delay_line {
            200 * 64
        } else {
            200
        } * 48000
            / 1000
            + 42;
        if self.shifter_l.delay_line.len() != delay_line_size {
            self.shifter_l.delay_line = DelayLine::new(delay_line_size);
            self.shifter_r.delay_line = DelayLine::new(delay_line_size);
        }
    }

    #[wasm_bindgen]
    pub fn process(&mut self, buffer: &mut DspBuffer) {
        let (left, right) = buffer.as_channels();
        self.shifter_l.process(left, self.use_larger_delay_line);
        self.shifter_r.process(right, self.use_larger_delay_line);
    }
}

#[derive(Default)]
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
#[derive(Default)]
struct Flanger {
    // not actually a SamplePair. left side is delayed input, right side is delayed output.
    delay_line: DelayLine<SamplePair>,

    interpolator: Interpolator<FlangerParams>,
}
impl Flanger {
    fn process(&mut self, buf: &mut [f32], use_larger_delay_line: bool) {
        for sample in &mut *buf {
            let params = self.interpolator.next();

            let (dx, dy) = if (params.voices - 1.0).abs() < 1e-5 {
                let SamplePair { l, r } = self.delay_line.compute(params.delay);
                (l, r)
            } else {
                let mut result = 0.0;
                let num_voices_int = params.voices.ceil() as usize;
                let delay_scale = if use_larger_delay_line {
                    1.0
                } else {
                    params.voices.recip().min(1.0)
                };
                for i in 1..=num_voices_int {
                    let SamplePair { l: mut val, .. } = self
                        .delay_line
                        .compute(params.delay * delay_scale * i as f32);
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
