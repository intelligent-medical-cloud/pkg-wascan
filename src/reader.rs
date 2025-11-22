use std::cell::RefCell;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, Event, HtmlInputElement};

use crate::{
    detector::detect_from_image,
    error::Error,
    event::{invoke_on_detect, invoke_on_start, invoke_on_stop},
};

const HIDDEN_FILE_INPUT_ID: &str = "wascan-file-input";

thread_local! {
    static DOCUMENT_REF: RefCell<Option<Document>> = const { RefCell::new(None) };
    static FILE_INPUT_READY: RefCell<bool> = const { RefCell::new(false) };
}

fn handle_detection_error(error: Error) {
    invoke_on_detect(Err(&error));
    invoke_on_stop();
}

pub fn init_reader() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from(Error::WindowNotFound))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from(Error::DocumentNotFound))?;

    DOCUMENT_REF.with(|doc_ref| {
        *doc_ref.borrow_mut() = Some(document.clone());
    });

    ensure_file_input(&document)?;

    FILE_INPUT_READY.with(|ready| {
        *ready.borrow_mut() = true;
    });

    Ok(())
}

fn ensure_file_input(doc: &Document) -> Result<(), JsValue> {
    let existing = doc.get_element_by_id(HIDDEN_FILE_INPUT_ID);
    if existing.is_some() {
        return Ok(());
    }

    let input = doc.create_element("input")?;
    input.set_attribute("type", "file")?;
    input.set_attribute("id", HIDDEN_FILE_INPUT_ID)?;
    input.set_attribute("accept", "image/*")?;
    input.set_attribute("style", "display: none;")?;

    if let Some(body) = doc.body() {
        body.append_child(&input)?;
    }

    let document_for_change = doc.clone();
    let change_closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_evt: Event| {
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

    if let Some(file_input) = doc.get_element_by_id(HIDDEN_FILE_INPUT_ID) {
        file_input
            .add_event_listener_with_callback("change", change_closure.as_ref().unchecked_ref())?;
        change_closure.forget();
    }

    Ok(())
}

pub fn read_from_image() -> Result<(), JsValue> {
    if !FILE_INPUT_READY.with(|ready| *ready.borrow()) {
        init_reader()?;
    }

    DOCUMENT_REF.with(|doc_ref| {
        if let Some(doc) = doc_ref.borrow().as_ref() {
            if let Some(input_el) = doc.get_element_by_id(HIDDEN_FILE_INPUT_ID)
                && let Ok(input_html) = input_el.dyn_into::<HtmlInputElement>()
            {
                input_html.click();

                Ok(())
            } else {
                Err(JsValue::from(Error::Internal))
            }
        } else {
            Err(JsValue::from(Error::DocumentNotFound))
        }
    })
}
