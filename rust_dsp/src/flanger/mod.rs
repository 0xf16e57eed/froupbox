// MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW MAOW

use wasm_bindgen::prelude::*;

mod flan;
pub use flan::{Flanger, FlangerParams};

use crate::{
    buffer::DspBuffer,
    delay_line::DelayLine,
    util::{self},
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
struct FlangerInstance {
    pub use_larger_delay_line: bool,

    shifter_l: Flanger<f32>,
    shifter_r: Flanger<f32>,
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
        for val in left {
            *val = self.shifter_l.process(*val, self.use_larger_delay_line);
        }
        for val in right {
            *val = self.shifter_r.process(*val, self.use_larger_delay_line);
        }
    }
}
