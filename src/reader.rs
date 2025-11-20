use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{Document, Element, Event, HtmlInputElement};

use crate::{
    detector::detect_from_image,
    error::Error,
    event::{invoke_on_detect, invoke_on_start, invoke_on_stop},
};

const HIDDEN_FILE_INPUT_ID: &str = "wascan-file-input";

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
            invoke_on_detect(Err(&Error::Internal));
            invoke_on_stop();

            return;
        };

        let Ok(input_html) = input_el.dyn_into::<HtmlInputElement>() else {
            invoke_on_detect(Err(&Error::Internal));
            invoke_on_stop();

            return;
        };

        let Some(files) = input_html.files() else {
            invoke_on_detect(Err(&Error::NoFileSelected));
            invoke_on_stop();

            return;
        };

        let Some(file) = files.get(0) else {
            invoke_on_detect(Err(&Error::NoFileSelected));
            invoke_on_stop();

            return;
        };

        if !file.type_().starts_with("image/") {
            invoke_on_detect(Err(&Error::InvalidMime));
            invoke_on_stop();

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
