mod detector;
mod error;
mod event;
mod reader;
mod scanner;

use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

/// WASM entry point
#[wasm_bindgen(start)]
pub fn main_js() {
    console_error_panic_hook::set_once();
}

/// Initializes the reader module. Must be called before using `read_from_image`.
#[wasm_bindgen]
pub fn init_reader() -> Result<(), JsValue> {
    reader::init_reader()
}

/// Initializes the scanner module. Must be called before using `start_stream_scan`.
#[wasm_bindgen]
pub fn init_scanner() -> Result<(), JsValue> {
    scanner::init_scanner()
}

/// Triggers the file input dialog to read an image file.
#[wasm_bindgen]
pub fn read_from_image() -> Result<(), JsValue> {
    reader::read_from_image()
}

/// Starts the stream-based barcode scanning from the camera.
///
/// ## Arguments
/// * `video_element_id` - The ID of the video element in the DOM where the stream will be displayed
#[wasm_bindgen]
pub fn start_stream_scan(video_element_id: &str) -> Result<(), JsValue> {
    scanner::start_stream_scan(video_element_id)
}

/// Stops the stream scanning programmatically.
#[wasm_bindgen]
pub fn stop_stream_scan() {
    scanner::stop_stream_scan();
}
