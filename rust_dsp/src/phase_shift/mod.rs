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
pub struct PhaseShiftInstanceParams {
    pub delay: f32,
    pub panning: f32,
    pub mix: f32,
    pub feedmix: f32,
}
impl PhaseShiftInstanceParams {
    fn split(&self, sample_rate: f32) -> (PhaseShifterParams, PhaseShifterParams) {
        let delay = self.delay * 0.000024414063 * sample_rate;
        let panning = self.panning * 0.5 + 0.5;
        let mix = self.mix * (1.0 / 63.0);
        let feedmix = self.feedmix * (1.0 / 64.0);
        (
            PhaseShifterParams {
                delay: delay * (1.0 - panning),
                mix,
                feedmix,
            },
            PhaseShifterParams {
                delay: delay * panning,
                mix,
                feedmix,
            },
        )
    }
}

#[wasm_bindgen]
#[derive(Default)]
pub struct PhaseShiftInstance {
    pub start: PhaseShiftInstanceParams,
    pub end: PhaseShiftInstanceParams,

    shifter_l: PhaseShifter,
    shifter_r: PhaseShifter,
}
#[wasm_bindgen]
impl PhaseShiftInstance {
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

struct PhaseShifterParams {
    delay: f32,
    mix: f32,
    feedmix: f32,
}
impl Zippable for PhaseShifterParams {
    fn zip(&self, other: &Self, f: impl Fn(f32, f32) -> f32) -> Self {
        Self {
            delay: f(self.delay, other.delay),
            mix: f(self.mix, other.mix),
            feedmix: f(self.feedmix, other.feedmix),
        }
    }
}

/// mono phase shifter using linear interpolation
struct PhaseShifter {
    // not actually a SamplePair. left side is delayed input, right side is delayed output.
    delay_line: DelayLine<SamplePair>,
}
impl Default for PhaseShifter {
    fn default() -> Self {
        Self {
            // 200ms at 48kHz
            delay_line: DelayLine::new(200 * (48000 / 1000) + 42),
        }
    }
}
impl PhaseShifter {
    fn process(&mut self, buf: &mut [f32], mut interpolator: Interpolator<PhaseShifterParams>) {
        for sample in buf {
            let params = interpolator.next();
            let SamplePair { l: dx, r: dy } = self.delay_line.compute(params.delay);
            let x = *sample;
            let y = lerp(x, lerp(dx, dy, params.feedmix), params.mix);
            self.delay_line.push(SamplePair { l: x, r: y });
            *sample = y;
        }
    }
}
