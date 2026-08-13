#[derive(Default)]
pub struct LegacyPhaser {
    inner: super::SimdPhaserWrapper<super::unipole::PhaserStage>,
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
        let coef = self.inner.i_break_coef.next();
        let mut wet = dry;
        for i in 0..self.inner.num_stages {
            self.inner.compute_one(i, &coef, &mut wet);
        }
        (dry, wet)
    }
}
