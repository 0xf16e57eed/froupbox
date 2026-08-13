//! Phaser using 1-pole all-pass filters (same as phaser in other Beepmods.)

use core::simd::f32x4;

#[derive(Clone, Copy, Default)]
pub struct PhaserStage {
    prev_input: f32x4,
    output: f32x4,
}
impl super::SimdPhaserStage for PhaserStage {
    type Coefficients = f32;
    type SimdCoefficients = f32x4;

    fn calculate_coefficients(freq: f32, _q: f32, sample_rate: f32) -> f32 {
        let break_t = f32::tan(0.5 * crate::filters::to_w0(freq, sample_rate));
        (break_t - 1.0) / (break_t + 1.0)
    }
    fn simd_coefficients_from_array(interp: &[f32; 4]) -> f32x4 {
        f32x4::from_array(*interp)
    }

    fn compute(&mut self, break_coef: &f32x4, input: &mut f32x4) {
        self.output = *break_coef * (*input - self.output) + self.prev_input;
        self.prev_input = *input;
        *input = self.output;
    }
    fn compute_single(&mut self, idx: usize, break_coef: &f32, input: &mut f32) {
        self.output.as_mut_array()[idx] = *break_coef * (*input - self.output.as_mut_array()[idx])
            + self.prev_input.as_mut_array()[idx];
        self.prev_input.as_mut_array()[idx] = *input;
        *input = self.output.as_mut_array()[idx];
    }

    fn concat_rotate(&mut self, other: &mut Self) {
        crate::util::concat_rotate(&mut self.prev_input, &mut other.prev_input);
        crate::util::concat_rotate(&mut self.output, &mut other.output);
    }
}
