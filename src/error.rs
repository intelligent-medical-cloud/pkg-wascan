use js_sys::{Object, Reflect};
use thiserror::Error;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

/// Error types for the wascan library.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error, Clone, Copy)]
pub enum Error {
    #[error("ERR_NO_WINDOW")]
    WindowNotFound,

    #[error("ERR_NO_DOCUMENT")]
    DocumentNotFound,

    #[error("ERR_NO_FILE_SELECTED")]
    NoFileSelected,

    #[error("ERR_INVALID_MIME")]
    InvalidMime,

    #[error("ERR_INVALID_VIDEO_ELEMENT_ID")]
    InvalidVideoElementId,

    #[error("ERR_NO_MEDIA")]
    NoMedia,

    #[error("ERR_NO_PERMISSION")]
    NoPermission,

    #[error("ERR_NOT_DETECTED")]
    NotDetected,

    #[error("ERR_INTERNAL")]
    Internal,
}

impl Error {
    pub fn code(&self) -> &'static str {
        match self {
            Error::WindowNotFound => "ERR_NO_WINDOW",
            Error::DocumentNotFound => "ERR_NO_DOCUMENT",
            Error::NoFileSelected => "ERR_NO_FILE_SELECTED",
            Error::InvalidMime => "ERR_INVALID_MIME",
            Error::InvalidVideoElementId => "ERR_INVALID_VIDEO_ELEMENT_ID",
            Error::NoMedia => "ERR_NO_MEDIA",
            Error::NoPermission => "ERR_NO_PERMISSION",
            Error::NotDetected => "ERR_NOT_DETECTED",
            Error::Internal => "ERR_INTERNAL",
        }
    }
}

impl From<Error> for JsValue {
    fn from(error: Error) -> Self {
        JsValue::from_str(error.code())
    }
}

fn set_str(obj: &Object, key: &str, value: &str) -> Result<(), JsValue> {
    Reflect::set(obj, &JsValue::from_str(key), &JsValue::from_str(value))
        .map_err(|_| JsValue::from_str("Failed to set property"))
        .and_then(|success| {
            if success {
                Ok(())
            } else {
                Err(JsValue::from_str("Failed to set property"))
            }
        })
}

#[wasm_bindgen]
pub fn error_codes() -> JsValue {
    let obj = Object::new();
    let errors = [
        ("WindowNotFound", Error::WindowNotFound),
        ("DocumentNotFound", Error::DocumentNotFound),
        ("NoFileSelected", Error::NoFileSelected),
        ("InvalidMime", Error::InvalidMime),
        ("InvalidVideoElementId", Error::InvalidVideoElementId),
        ("NoMedia", Error::NoMedia),
        ("NoPermission", Error::NoPermission),
        ("NotDetected", Error::NotDetected),
        ("Internal", Error::Internal),
    ];

    for (key, error) in errors {
        if set_str(&obj, key, error.code()).is_err() {
            continue;
        }
    }

    obj.into()
}

pub fn error_to_js(error: &Error) -> JsValue {
    JsValue::from(*error)
}
