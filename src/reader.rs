use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{Document, Element, Event, HtmlInputElement};

use crate::{
    detector::detect_from_image,
    error::Error,
    event::{invoke_on_detect, invoke_on_start, invoke_on_stop},
};

const HIDDEN_FILE_INPUT_ID: &str = "wascan-file-input";

/// Helper function to handle detection errors consistently.
fn handle_detection_error(error: Error) {
    invoke_on_detect(Err(&error));
    invoke_on_stop();
}

/// Initializes the file reader for image-based barcode detection.
///
/// Sets up a hidden file input element and connects it to the provided button.
/// When a file is selected, it will be processed for barcode detection.
pub fn init_reader(document: &Document, button: &Element) -> Result<(), JsValue> {
    let existing = document.get_element_by_id(HIDDEN_FILE_INPUT_ID);
    let file_input = if let Some(el) = existing {
        el
    } else {
        let input = document.create_element("input")?;
        input.set_attribute("type", "file")?;
        input.set_attribute("id", HIDDEN_FILE_INPUT_ID)?;
        input.set_attribute("accept", "image/*")?;
        input.set_attribute("style", "display: none;")?;
        if let Some(body) = document.body() {
            body.append_child(&input)?;
        }

        input
    };

    let document_for_change = document.clone();
    let change_closure = Closure::wrap(Box::new(move |_evt: Event| {
        invoke_on_start();

        let Some(input_el) = document_for_change.get_element_by_id(HIDDEN_FILE_INPUT_ID) else {
            handle_detection_error(Error::Internal);
            return;
        };

        let Ok(input_html) = input_el.dyn_into::<HtmlInputElement>() else {
            handle_detection_error(Error::Internal);
            return;
        };

        let Some(files) = input_html.files() else {
            handle_detection_error(Error::NoFileSelected);
            return;
        };

        let Some(file) = files.get(0) else {
            handle_detection_error(Error::NoFileSelected);
            return;
        };

        if !file.type_().starts_with("image/") {
            handle_detection_error(Error::InvalidMime);
            return;
        }

        detect_from_image(file);
    }) as Box<dyn FnMut(_)>);
    file_input
        .add_event_listener_with_callback("change", change_closure.as_ref().unchecked_ref())?;
    change_closure.forget();

    let document_for_click = document.clone();
    let click_closure = Closure::wrap(Box::new(move |_evt: Event| {
        if let Some(input_el) = document_for_click.get_element_by_id(HIDDEN_FILE_INPUT_ID)
            && let Ok(input_html) = input_el.dyn_into::<HtmlInputElement>()
        {
            input_html.click();
        }
    }) as Box<dyn FnMut(_)>);
    button.add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())?;
    click_closure.forget();

    Ok(())
}
