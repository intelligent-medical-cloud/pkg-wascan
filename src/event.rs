use std::cell::RefCell;

use js_sys::{Function, Object, Reflect};
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use web_sys::console;

use crate::error::{self, Error};

thread_local! {
    static ON_START: RefCell<Option<Function>> = const { RefCell::new(None) };
    static ON_DETECT: RefCell<Option<Function>> = const { RefCell::new(None) };
    static ON_STOP: RefCell<Option<Function>> = const { RefCell::new(None) };
}

/// Invokes the on_start callback if one is registered.
pub fn invoke_on_start() {
    ON_START.with(|slot| {
        if let Some(cb) = &*slot.borrow() {
            let res = cb.call1(&JsValue::NULL, &JsValue::NULL);
            if let Err(e) = res {
                console::error_1(&e);
            }
        }
    });
}

/// Invokes the on_detect callback with the detection result.
///
/// # Arguments
///
/// * `result` - Either a successful detection string or an error reference.
pub fn invoke_on_detect(result: Result<&str, &Error>) {
    let cb_arg = {
        let obj = Object::new();
        match result {
            Ok(value) => {
                let _ = Reflect::set(
                    &obj,
                    &JsValue::from_str("success"),
                    &JsValue::from_bool(true),
                );
                let _ = Reflect::set(&obj, &JsValue::from_str("value"), &JsValue::from_str(value));
                obj.into()
            }
            Err(error) => {
                let _ = Reflect::set(
                    &obj,
                    &JsValue::from_str("success"),
                    &JsValue::from_bool(false),
                );
                let _ = Reflect::set(
                    &obj,
                    &JsValue::from_str("error"),
                    &error::error_to_js(error),
                );
                obj.into()
            }
        }
    };

    ON_DETECT.with(|slot| {
        if let Some(cb) = &*slot.borrow() {
            let res = cb.call1(&JsValue::NULL, &cb_arg);
            if let Err(e) = res {
                console::error_1(&e);
            }
        }
    });
}

/// Invokes the on_stop callback if one is registered.
pub fn invoke_on_stop() {
    ON_STOP.with(|slot| {
        if let Some(cb) = &*slot.borrow() {
            let res = cb.call1(&JsValue::NULL, &JsValue::NULL);
            if let Err(e) = res {
                console::error_1(&e);
            }
        }
    });
}

/// Registers a callback function to be called when scanning starts.
#[wasm_bindgen]
pub fn on_start(cb: Function) {
    ON_START.with(|slot| *slot.borrow_mut() = Some(cb));
}

/// Registers a callback function to be called when a barcode is detected.
///
/// The callback receives an object with:
/// - `success: boolean` - true if detection succeeded, false otherwise
/// - `value?: string` - the detected barcode (only present if success is true)
/// - `error?: string` - the error code (only present if success is false)
#[wasm_bindgen]
pub fn on_detect(cb: Function) {
    ON_DETECT.with(|slot| *slot.borrow_mut() = Some(cb));
}

/// Registers a callback function to be called when scanning stops.
#[wasm_bindgen]
pub fn on_stop(cb: Function) {
    ON_STOP.with(|slot| *slot.borrow_mut() = Some(cb));
}
