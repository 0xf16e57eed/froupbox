#[derive(Default)]
pub struct LegacyPhaser {
    inner: super::SimdPhaserWrapper<super::unipole::PhaserStage>,
    prev_output: f32,
}

impl super::PhaserAlgorithm for LegacyPhaser {
    fn begin(
        &mut self,
        start: super::PhaserInstanceParams,
        end: super::PhaserInstanceParams,
        sample_rate: f32,
        run_length: f32,
    ) {
        self.inner.begin(start, end, sample_rate, run_length);
    }
    fn resize(&mut self, num_stages: usize) {
        self.inner.resize(num_stages);
    }
    fn compute(&mut self, dry: f32) -> (f32, f32) {
        let coef = self.inner.next_coef();
        let mut wet = dry + self.prev_output * self.inner.i_feedback_mult.next();
        for i in 0..self.inner.num_stages {
            self.inner.compute_one(i, &coef, &mut wet);
        }
        self.prev_output = wet;
        (dry, wet)
    }
}
