#![feature(portable_simd)]

mod buffer;
mod compressor;
mod delay_line;
mod filters;
mod phase_shift;
mod phaser;
mod sample;
mod util;

pub(crate) use sample::{Sample, SamplePair, lerp};

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub(crate) fn log(s: String);
}
