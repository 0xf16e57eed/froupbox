//! https://gitlab.com/cyphers-stuff/cybox/-/blob/31c2eda59748f321a09141b41552d8e65a755dfe/dsp/beepbox/src/filters.rs

use core::f32;

use crate::SamplePair;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FilterDirection {
    Low,
    High,
}

/// angular frequency, but without the `TAU` factor. i.e. it's just `freq / sample_rate`
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default)]
pub struct AngularFrequency(pub f32);
impl AngularFrequency {
    pub fn new(freq: f32, sample_rate: f32) -> Self {
        Self(freq / sample_rate)
    }
    pub fn sin_cos(self) -> (f32, f32) {
        // like `cybox_util::osc::sine`, but only valid from `0..0.5`.
        fn sin_half(x: f32) -> f32 {
            debug_assert!((0.0..=0.5).contains(&x), "{x} out of range for sin_half");
            let x = x * (0.5 - x);
            12.4 * x + 57.6 * x * x
        }
        let sin = sin_half(self.0);
        let mut unsigned_cos = (1.0 - sin * sin).sqrt();
        if self.0 > 0.25 {
            unsigned_cos = -unsigned_cos;
        }
        (sin, unsigned_cos)
    }
    pub fn tan(self) -> f32 {
        let (sin, cos) = self.sin_cos();
        sin / cos
    }
}
impl core::ops::Mul<f32> for AngularFrequency {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self(self.0 * rhs)
    }
}
impl crate::util::Zippable for AngularFrequency {
    fn zip(&self, other: &Self, f: impl Fn(f32, f32) -> f32) -> Self {
        Self(f(self.0, other.0))
    }
}

#[derive(Debug, Clone, Default)]
pub struct BiquadFilterCoefficients {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a1: f32,
    pub a2: f32,
}
impl BiquadFilterCoefficients {
    pub fn new(b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) -> Self {
        let inv_a0 = a0.recip();
        Self {
            b0: b0 * inv_a0,
            b1: b1 * inv_a0,
            b2: b2 * inv_a0,
            a1: a1 * inv_a0,
            a2: a2 * inv_a0,
        }
    }

    pub fn pass(dir: FilterDirection, w0: AngularFrequency, mult: f32) -> Self {
        let (sw0, cw0) = w0.sin_cos();

        let alpha = sw0 / mult;

        let bmult = match dir {
            FilterDirection::Low => 1.0,
            FilterDirection::High => -1.0,
        };
        let b1 = bmult - cw0;

        Self::new(
            bmult * b1,
            2.0 * b1,
            bmult * b1,
            2.0 + alpha,
            -4.0 * cw0,
            2.0 - alpha,
        )
    }
}

#[derive(Default)]
pub struct BiquadFilterValues {
    v1: SamplePair,
    v2: SamplePair,
}
impl BiquadFilterValues {
    pub fn run(&mut self, c: &BiquadFilterCoefficients, x0: SamplePair) -> SamplePair {
        let v0 = x0 - c.a1 * self.v1 - c.a2 * self.v2;
        let output = v0 * c.b0 + self.v1 * c.b1 + self.v2 * c.b2;
        self.v2 = self.v1;
        self.v1 = v0;
        output
    }
}

#[derive(Clone, Default)]
pub struct CrossoverCoefficients {
    pub lo: BiquadFilterCoefficients,
    pub hi: BiquadFilterCoefficients,
}
impl CrossoverCoefficients {
    pub fn new(w0: AngularFrequency) -> Self {
        Self {
            lo: BiquadFilterCoefficients::pass(
                FilterDirection::Low,
                w0,
                f32::consts::FRAC_1_SQRT_2,
            ),
            hi: BiquadFilterCoefficients::pass(
                FilterDirection::High,
                w0,
                f32::consts::FRAC_1_SQRT_2,
            ),
        }
    }
}

/// A [Linkwitz-Riley](https://en.wikipedia.org/wiki/Linkwitz%E2%80%93Riley_filter) crossover to split an audio signal at a given frequency.
#[derive(Default)]
pub struct Crossover {
    pub l0: BiquadFilterValues,
    pub l1: BiquadFilterValues,
    pub h0: BiquadFilterValues,
    pub h1: BiquadFilterValues,
}
impl Crossover {
    pub fn run(&mut self, coef: &CrossoverCoefficients, lo: &mut SamplePair, hi: &mut SamplePair) {
        *lo = self.l0.run(&coef.lo, *lo);
        *lo = self.l1.run(&coef.lo, *lo);
        *hi = self.h0.run(&coef.hi, *hi);
        *hi = self.h1.run(&coef.hi, *hi);
    }
}
