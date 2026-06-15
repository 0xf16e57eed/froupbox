#![feature(portable_simd)]

mod compressor;
mod filters;
mod phaser;
mod sample;
mod util;

pub(crate) use sample::{SamplePair, lerp};

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}
