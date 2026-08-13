//! Phaser using 2-pole all-pass filters.

use core::simd::f32x4;

use crate::filters::AngularFrequency;

#[derive(Clone, Copy, Default)]
pub struct PhaserStage {
    x1: f32x4,
    x2: f32x4,
    y1: f32x4,
    y2: f32x4,
}
impl super::SimdPhaserStage for PhaserStage {
    type Coefficients = BreakCoefficients;
    type SimdCoefficients = SimdBreakCoefficients;

    fn calculate_coefficients(w0: AngularFrequency, q: f32) -> BreakCoefficients {
        let (sin, cos) = w0.sin_cos();
        let alpha = sin / (2.0 * q);
        BreakCoefficients((1.0 - alpha) / (1.0 + alpha), (2.0 * cos) / (1.0 + alpha))
    }
    fn simd_coefficients_from_array(interp: &[BreakCoefficients; 4]) -> SimdBreakCoefficients {
        let mut res = SimdBreakCoefficients::default();
        for (i, coef) in interp.iter().enumerate() {
            res.0.as_mut_array()[i] = coef.0;
            res.1.as_mut_array()[i] = coef.1;
        }
        res
    }

    fn compute_simd(&mut self, break_coef: &SimdBreakCoefficients, input: &mut f32x4) {
        // y0 = ((1-a)/(1+a)) (x0 - y2) + 2cosw0/(1+a) (y1 - x1) + x2
        let output =
            break_coef.0 * (*input - self.y2) + break_coef.1 * (self.y1 - self.x1) + self.x2;
        self.x2 = self.x1;
        self.x1 = *input;
        self.y2 = self.y1;
        self.y1 = output;
        *input = output;
    }
    fn compute(&mut self, idx: usize, break_coef: &BreakCoefficients, input: &mut f32) {
        let output = break_coef.0 * (*input - self.y2.as_mut_array()[idx])
            + break_coef.1 * (self.y1.as_mut_array()[idx] - self.x1.as_mut_array()[idx])
            + self.x2.as_mut_array()[idx];
        self.x2.as_mut_array()[idx] = self.x1.as_mut_array()[idx];
        self.x1.as_mut_array()[idx] = *input;
        self.y2.as_mut_array()[idx] = self.y1.as_mut_array()[idx];
        self.y1.as_mut_array()[idx] = output;
        *input = output;
    }

    fn concat_rotate(&mut self, other: &mut Self) {
        crate::util::concat_rotate(&mut self.x1, &mut other.x1);
        crate::util::concat_rotate(&mut self.y1, &mut other.y1);
        crate::util::concat_rotate(&mut self.x2, &mut other.x2);
        crate::util::concat_rotate(&mut self.y2, &mut other.y2);
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BreakCoefficients(
    // (1-a)/(1+a)
    f32,
    // 2cosw0/(1+a)
    f32,
);
impl crate::util::Zippable for BreakCoefficients {
    fn zip(&self, other: &Self, f: impl Fn(f32, f32) -> f32) -> Self {
        Self(f(self.0, other.0), f(self.1, other.1))
    }
}

#[derive(Default, Clone, Copy)]
pub struct SimdBreakCoefficients(
    // (1-a)/(1+a)
    f32x4,
    // 2cosw0/(1+a)
    f32x4,
);
