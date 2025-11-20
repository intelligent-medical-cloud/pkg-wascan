mod config;
mod detector;
mod error;
mod event;
mod reader;
mod scanner;

use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

use error::Error;

// WASM ENTRY POINT
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = web_sys::window().ok_or(JsValue::from(Error::WindowNotFound.to_string()))?;
    let document = window
        .document()
        .ok_or(JsValue::from(Error::DocumentNotFound.to_string()))?;

    let mut read_available = false;
    let mut scan_available = false;
    let mut _stop_available = false;

    let read_from_image_button = document.get_element_by_id(config::READ_FROM_IMAGE_BUTTON_ID);
    if read_from_image_button.is_some() {
        read_available = true;
    }

    let scan_from_stream_button = document.get_element_by_id(config::SCAN_FROM_STREAM_BUTTON_ID);
    if scan_from_stream_button.is_some() {
        scan_available = true;
    }

    let stop_stream_scan_button = document.get_element_by_id(config::STOP_STREAM_SCAN_BUTTON_ID);
    if stop_stream_scan_button.is_some() {
        _stop_available = true;
    }

    if !read_available && !scan_available {
        return Err(JsValue::from(Error::TriggerButtonsNotFound.to_string()));
    }

    if read_available {
        reader::init_reader(&document, &read_from_image_button.unwrap())?;
    }

    Ok(())
}
