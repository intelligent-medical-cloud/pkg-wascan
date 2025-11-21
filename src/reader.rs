use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{Document, Event, HtmlInputElement};

use crate::{
    config,
    detector::detect_from_image,
    error::Error,
    event::{invoke_on_detect, invoke_on_start, invoke_on_stop},
};

const HIDDEN_FILE_INPUT_ID: &str = "wascan-file-input";

static CLICK_DELEGATION_SETUP: AtomicBool = AtomicBool::new(false);
static CHANGE_LISTENER_SETUP: AtomicBool = AtomicBool::new(false);

thread_local! {
    static DOCUMENT_REF: RefCell<Option<Document>> = const { RefCell::new(None) };
}

fn handle_detection_error(error: Error) {
    invoke_on_detect(Err(&error));
    invoke_on_stop();
}

/// Initializes the file reader for image-based barcode detection.
///
/// Uses event delegation on the document level, so it works even if buttons
/// are dynamically removed and re-added by frameworks like Vue/React.
/// This function is idempotent and can be called multiple times safely.
pub fn init_reader(doc: &Document) -> Result<(), JsValue> {
    DOCUMENT_REF.with(|doc_ref| {
        *doc_ref.borrow_mut() = Some(doc.clone());
    });

    ensure_file_input(doc)?;

    if !CLICK_DELEGATION_SETUP.swap(true, Ordering::Acquire) {
        setup_click_delegation(doc)?;
    }

    if !CHANGE_LISTENER_SETUP.swap(true, Ordering::Acquire) {
        setup_change_listener(doc)?;
    }

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

    Ok(())
}

fn setup_click_delegation(doc: &Document) -> Result<(), JsValue> {
    let click_closure = Closure::wrap(Box::new(move |evt: Event| {
        let Some(target) = evt.target() else {
            return;
        };

        let Some(element) = target.dyn_ref::<web_sys::Element>() else {
            return;
        };

        let element_id = element.id();
        if element_id == HIDDEN_FILE_INPUT_ID {
            return;
        }

        if element_id == config::READ_FROM_IMAGE_BUTTON_ID
            && let Some(window) = web_sys::window()
        {
            let click_closure = Closure::wrap(Box::new(move || {
                DOCUMENT_REF.with(|doc_ref| {
                    if let Some(doc) = doc_ref.borrow().as_ref()
                        && let Some(input_el) = doc.get_element_by_id(HIDDEN_FILE_INPUT_ID)
                        && let Ok(input_html) = input_el.dyn_into::<HtmlInputElement>()
                    {
                        input_html.click();
                    }
                });
            }) as Box<dyn FnMut()>);

            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                click_closure.as_ref().unchecked_ref(),
                0,
            );
            click_closure.forget();
        }
    }) as Box<dyn FnMut(_)>);

    doc.add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())?;
    click_closure.forget();

    Ok(())
}

fn setup_change_listener(doc: &Document) -> Result<(), JsValue> {
    let document_for_change = doc.clone();
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

    let Some(file_input) = doc.get_element_by_id(HIDDEN_FILE_INPUT_ID) else {
        return Err(JsValue::from(Error::Internal));
    };

    file_input
        .add_event_listener_with_callback("change", change_closure.as_ref().unchecked_ref())?;
    change_closure.forget();

    Ok(())
}
