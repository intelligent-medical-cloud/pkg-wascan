mod config;
mod detector;
mod error;
mod event;
mod reader;
mod scanner;

use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

use error::Error;

/// WASM entry point
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = web_sys::window()
        .ok_or_else(|| JsValue::from(Error::WindowNotFound))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from(Error::DocumentNotFound))?;

    let read_from_image_button = document.get_element_by_id(config::READ_FROM_IMAGE_BUTTON_ID);
    let scan_from_stream_button = document.get_element_by_id(config::SCAN_FROM_STREAM_BUTTON_ID);

    if read_from_image_button.is_none() && scan_from_stream_button.is_none() {
        return Err(JsValue::from(Error::TriggerButtonsNotFound));
    }

    if let Some(button) = read_from_image_button {
        reader::init_reader(&document, &button)?;
    }

    // TODO: Initialize stream scanner when scan_from_stream_button is available

    Ok(())
}
