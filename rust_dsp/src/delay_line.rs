// https://gitlab.com/cyphers-stuff/cybox/-/blob/31c2eda59748f321a09141b41552d8e65a755dfe/dsp/beepbox/src/delay_line.rs
use crate::{Sample, SamplePair};

pub struct DelayLine<T: Sample = SamplePair> {
    index: usize,
    silence_counter: SilenceCounter,
    pub buf: Box<[T]>,
}

pub const SILENCE_SAMPLE_THRESHOLD: f32 = 1e-4;
const SILENCE_COUNTER_THRESHOLD: usize = 44100;

impl<T: Sample> DelayLine<T> {
    pub fn new(length: usize) -> Self {
        Self {
            index: 0,
            silence_counter: Default::default(),
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
        self.silence_counter.process(pair);
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
    pub fn is_silent(&self) -> bool {
        self.silence_counter.is_silent()
    }
}

#[derive(Default)]
pub struct SilenceCounter {
    count: usize,
}
impl SilenceCounter {
    pub fn process<T: Sample>(&mut self, sanitized: T) {
        if sanitized.is_silent_below(SILENCE_SAMPLE_THRESHOLD) {
            self.count += 1;
        } else {
            self.count = 0;
        }
    }
    pub fn is_silent(&self) -> bool {
        self.count >= SILENCE_COUNTER_THRESHOLD
    }
}
