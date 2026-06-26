use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Default)]
pub struct DspBuffer {
    buffer: Box<[f32]>,
    pub sample_rate: f32,
    pub run_length: usize,
}
#[wasm_bindgen]
impl DspBuffer {
    #[wasm_bindgen(constructor)]
    pub fn new(frame_size: usize) -> Self {
        Self {
            buffer: vec![0.0; frame_size * 2].into_boxed_slice(),
            ..Default::default()
        }
    }
    #[wasm_bindgen(getter)]
    pub fn buffer(&mut self) -> js_sys::Float32Array {
        unsafe { js_sys::Float32Array::view(&self.buffer) }
    }
}
impl DspBuffer {
    pub fn as_channels(&mut self) -> (&mut [f32], &mut [f32]) {
        let (left, right) = self.buffer.split_at_mut(self.buffer.len() / 2);
        (&mut left[..self.run_length], &mut right[..self.run_length])
    }
    pub fn as_zipped(&mut self) -> impl Iterator<Item = (&mut f32, &mut f32)> {
        let (left, right) = self.as_channels();
        std::iter::zip(left, right)
    }
}
