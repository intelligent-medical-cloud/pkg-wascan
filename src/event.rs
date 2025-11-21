use std::cell::RefCell;

use js_sys::Function;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use web_sys::console;

use crate::error::{self, Error};

thread_local! {
    static ON_START: RefCell<Option<Function>> = const { RefCell::new(None) };
    static ON_DETECT: RefCell<Option<Function>> = const { RefCell::new(None) };
    static ON_STOP: RefCell<Option<Function>> = const { RefCell::new(None) };
}

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

pub fn invoke_on_detect(result: Result<&str, &Error>) {
    let cb_arg = match result {
        Ok(v) => JsValue::from_str(v),
        Err(e) => error::error_to_js(e),
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

#[wasm_bindgen]
pub fn on_start(cb: Function) {
    ON_START.with(|slot| *slot.borrow_mut() = Some(cb));
}

#[wasm_bindgen]
pub fn on_detect(cb: Function) {
    ON_DETECT.with(|slot| *slot.borrow_mut() = Some(cb));
}

#[wasm_bindgen]
pub fn on_stop(cb: Function) {
    ON_STOP.with(|slot| *slot.borrow_mut() = Some(cb));
}
