use js_sys::{Object, Reflect};
use thiserror::Error;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error)]
pub enum Error {
    #[error("ERR_NO_WINDOW")]
    WindowNotFound,

    #[error("ERR_NO_DOCUMENT")]
    DocumentNotFound,

    #[error("ERR_NO_TRIGGER_BUTTONS")]
    TriggerButtonsNotFound,

    #[error("ERR_NO_FILE_SELECTED")]
    NoFileSelected,

    #[error("ERR_INVALID_MIME")]
    InvalidMime,

    #[error("ERR_NOT_DETECTED")]
    NotDetected,

    #[error("ERR_INTERNAL")]
    Internal,
}

#[wasm_bindgen]
pub fn error_codes() -> JsValue {
    let obj = Object::new();
    set_str(
        &obj,
        "WindowNotFound",
        Error::WindowNotFound.to_string().as_str(),
    );
    set_str(
        &obj,
        "DocumentNotFound",
        Error::DocumentNotFound.to_string().as_str(),
    );
    set_str(
        &obj,
        "TriggerButtonsNotFound",
        Error::TriggerButtonsNotFound.to_string().as_str(),
    );
    set_str(
        &obj,
        "NoFileSelected",
        Error::NoFileSelected.to_string().as_str(),
    );
    set_str(&obj, "InvalidMime", Error::InvalidMime.to_string().as_str());
    set_str(&obj, "NotDetected", Error::NotDetected.to_string().as_str());
    set_str(&obj, "Internal", Error::Internal.to_string().as_str());

    obj.into()
}

pub fn error_to_js(error: &Error) -> JsValue {
    let obj = Object::new();
    set_str(&obj, "message", &error.to_string());
    obj.into()
}

fn set_str(obj: &Object, key: &str, value: &str) {
    let _ = Reflect::set(obj, &JsValue::from_str(key), &JsValue::from_str(value));
}
