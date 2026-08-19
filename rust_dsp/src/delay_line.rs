// https://gitlab.com/cyphers-stuff/cybox/-/blob/31c2eda59748f321a09141b41552d8e65a755dfe/dsp/beepbox/src/delay_line.rs
use crate::{Sample, SamplePair};

#[derive(Default)]
pub struct DelayLine<T: Sample = SamplePair> {
    index: usize,
    pub buf: Box<[T]>,
}

pub const SILENCE_SAMPLE_THRESHOLD: f32 = 1e-4;

impl<T: Sample> DelayLine<T> {
    pub fn new(length: usize) -> Self {
        Self {
            index: 0,
            buf: unsafe { Box::new_zeroed_slice(length).assume_init() },
        }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }
    pub fn push(&mut self, pair: T) {
        if self.index == 0 {
            self.index = self.len();
        }
        self.index -= 1;
        let sanitized = pair.sanitize_finite();
        self.buf[self.index] = sanitized;
    }
    pub fn compute(&mut self, samples: f32) -> T {
        let trunced = samples as usize;
        if samples.is_sign_negative() || trunced >= self.len() {
            // out of bounds
            return T::ZERO;
        }
        fn wrap_once(val: usize, n: usize) -> usize {
            val.min(val.wrapping_sub(n))
        }

        let index = wrap_once(self.index + trunced, self.len());
        let next_index = wrap_once(self.index + trunced + 1, self.len());

        self.buf[index].lerp(self.buf[next_index], samples - trunced as f32)
    }
}
