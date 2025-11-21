mod config;
mod detector;
mod error;
mod event;
mod reader;
mod scanner;

use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

use error::Error;
use reader::init_reader;

/// WASM entry point
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = web_sys::window().ok_or_else(|| JsValue::from(Error::WindowNotFound))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from(Error::DocumentNotFound))?;

    init_reader(&document)?;

    // TODO: Initialize stream scanner when scan_from_stream_button is available

    Ok(())
}

/// Re-initializes the reader. Can be called from JavaScript/Vue lifecycle hooks
/// if buttons are added dynamically after WASM initialization.
#[wasm_bindgen]
pub fn reinit_reader() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from(Error::WindowNotFound))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from(Error::DocumentNotFound))?;

    init_reader(&document)
}
