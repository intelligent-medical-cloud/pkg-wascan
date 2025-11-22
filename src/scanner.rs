use std::cell::{Cell, RefCell};
use std::rc::Rc;

use js_sys::{Date, Function, Object, Reflect};
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{
    CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlVideoElement, MediaStream,
    MediaStreamConstraints, MediaStreamTrack,
};

use crate::{
    detector::detect_from_stream,
    error::Error,
    event::{invoke_on_detect, invoke_on_start, invoke_on_stop},
};

thread_local! {
    static DOCUMENT_REF: RefCell<Option<Document>> = const { RefCell::new(None) };
    static STREAMING: Cell<bool> = const { Cell::new(false) };
    static RUNNING_FLAG: RefCell<Option<Rc<Cell<bool>>>> = const { RefCell::new(None) };
    static VIDEO_ELEMENT_ID: RefCell<Option<String>> = const { RefCell::new(None) };
    static LAST_DETECTED_CODE: RefCell<Option<String>> = const { RefCell::new(None) };
    static DETECTION_COUNT: Cell<u32> = const { Cell::new(0) };
}

const REQUIRED_CONSECUTIVE_DETECTIONS: u32 = 3;

fn handle_detection_error(error: Error) {
    invoke_on_detect(Err(&error));
    invoke_on_stop();
}

fn now_millis() -> u64 {
    Date::now() as u64
}

pub fn init_scanner() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from(Error::WindowNotFound))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from(Error::DocumentNotFound))?;

    DOCUMENT_REF.with(|doc_ref| {
        *doc_ref.borrow_mut() = Some(document);
    });

    Ok(())
}

pub fn start_stream_scan(video_element_id: &str) -> Result<(), JsValue> {
    if STREAMING.with(|s| s.get()) {
        return Ok(());
    }

    let doc = DOCUMENT_REF.with(|doc_ref| doc_ref.borrow().clone());
    let Some(doc) = doc else {
        return Err(JsValue::from(Error::DocumentNotFound));
    };

    let Some(video_el) = doc
        .get_element_by_id(video_element_id)
        .and_then(|e| e.dyn_into::<HtmlVideoElement>().ok())
    else {
        return Err(JsValue::from(Error::InvalidVideoElementId));
    };

    STREAMING.with(|s| s.set(true));
    VIDEO_ELEMENT_ID.with(|id| {
        *id.borrow_mut() = Some(video_element_id.to_string());
    });

    invoke_on_start();

    spawn_local(async move {
        let Some(window) = web_sys::window() else {
            STREAMING.with(|s| s.set(false));

            handle_detection_error(Error::WindowNotFound);

            return;
        };

        let navigator = window.navigator();
        let media_devices = match navigator.media_devices() {
            Ok(md) => md,
            Err(_) => {
                STREAMING.with(|s| s.set(false));

                handle_detection_error(Error::NoMedia);

                return;
            }
        };

        let constraints = MediaStreamConstraints::new();
        let video_constraints = Object::new();
        Reflect::set(
            &video_constraints,
            &JsValue::from_str("facingMode"),
            &JsValue::from_str("environment"),
        )
        .ok();

        constraints.set_video(&video_constraints.into());

        let g_um = match media_devices.get_user_media_with_constraints(&constraints) {
            Ok(s) => s,
            Err(_) => {
                STREAMING.with(|s| s.set(false));

                handle_detection_error(Error::NoMedia);

                return;
            }
        };

        let stream_js = match JsFuture::from(g_um).await {
            Ok(s) => s,
            Err(err) => {
                STREAMING.with(|s| s.set(false));

                let err_name = Reflect::get(&err, &JsValue::from_str("name"))
                    .ok()
                    .and_then(|name_val| name_val.as_string());
                let error_type = match err_name.as_deref() {
                    Some("NotAllowedError") | Some("PermissionDeniedError") => Error::NoPermission,
                    _ => Error::NoMedia,
                };

                handle_detection_error(error_type);

                return;
            }
        };

        let stream: MediaStream = match stream_js.dyn_into() {
            Ok(s) => s,
            Err(_) => {
                STREAMING.with(|s| s.set(false));

                handle_detection_error(Error::NoMedia);

                return;
            }
        };

        video_el.set_src_object(Some(&stream));
        video_el.set_muted(true);
        video_el.play().ok();

        let canvas: HtmlCanvasElement = match doc.create_element("canvas") {
            Ok(el) => match el.dyn_into() {
                Ok(canvas) => canvas,
                Err(_) => {
                    STREAMING.with(|s| s.set(false));

                    handle_detection_error(Error::Internal);

                    return;
                }
            },
            Err(_) => {
                STREAMING.with(|s| s.set(false));

                handle_detection_error(Error::Internal);

                return;
            }
        };
        canvas.set_attribute("style", "display: none; ").ok();

        let context_options = Object::new();
        Reflect::set(
            &context_options,
            &JsValue::from_str("willReadFrequently"),
            &JsValue::from_bool(true),
        )
        .ok();

        let ctx: CanvasRenderingContext2d = {
            let get_context_fn = Reflect::get(&canvas, &JsValue::from_str("getContext"))
                .ok()
                .and_then(|v| Function::from(v).into());

            let ctx_result: Option<CanvasRenderingContext2d> = if let Some(f) = get_context_fn {
                if let Ok(ctx_js) =
                    f.call2(&canvas, &JsValue::from_str("2d"), &context_options.into())
                {
                    ctx_js.dyn_into::<CanvasRenderingContext2d>().ok()
                } else {
                    canvas
                        .get_context("2d")
                        .ok()
                        .flatten()
                        .and_then(|ctx| ctx.dyn_into::<CanvasRenderingContext2d>().ok())
                }
            } else {
                canvas
                    .get_context("2d")
                    .ok()
                    .flatten()
                    .and_then(|ctx| ctx.dyn_into::<CanvasRenderingContext2d>().ok())
            };

            match ctx_result {
                Some(ctx) => ctx,
                None => {
                    STREAMING.with(|s| s.set(false));
                    handle_detection_error(Error::Internal);
                    return;
                }
            }
        };

        let last_scan_ms = Rc::new(Cell::new(0u64));
        let running = Rc::new(Cell::new(true));

        RUNNING_FLAG.with(|flag| {
            *flag.borrow_mut() = Some(running.clone());
        });

        let video_for_raf = video_el.clone();

        type RafCallback = Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>>;
        let raf_cb: RafCallback = Rc::new(RefCell::new(None));
        let raf_cb2 = raf_cb.clone();
        let running_clone = running.clone();
        let last_scan_ms_clone = last_scan_ms.clone();

        *raf_cb.borrow_mut() = Some(Closure::new(move |_ts: f64| {
            if !running_clone.get() {
                return;
            }

            let Some(window) = web_sys::window() else {
                return;
            };

            let now_ms = now_millis();
            if now_ms.saturating_sub(last_scan_ms_clone.get()) < 100 {
                if let Some(cb) = raf_cb2.borrow().as_ref() {
                    window
                        .request_animation_frame(cb.as_ref().unchecked_ref())
                        .ok();
                }

                return;
            }
            last_scan_ms_clone.set(now_ms);

            let vw = video_for_raf.video_width();
            let vh = video_for_raf.video_height();
            canvas.set_width(vw);
            canvas.set_height(vh);

            ctx.draw_image_with_html_video_element(&video_for_raf, 0.0, 0.0)
                .ok();
            let image_data = match ctx.get_image_data(0.0, 0.0, vw as f64, vh as f64) {
                Ok(d) => d,
                Err(_) => {
                    if let Some(cb) = raf_cb2.borrow().as_ref() {
                        window
                            .request_animation_frame(cb.as_ref().unchecked_ref())
                            .ok();
                    }

                    return;
                }
            };

            let data = image_data.data();
            let bytes: Vec<u8> = data.0;
            let mut gray = vec![0u8; (vw * vh) as usize];
            for (dst, px) in gray.iter_mut().zip(bytes.chunks_exact(4)) {
                let r = px[0] as f32;
                let g = px[1] as f32;
                let b = px[2] as f32;
                let y = 0.299 * r + 0.587 * g + 0.114 * b;
                *dst = y as u8;
            }

            if let Ok(text) = detect_from_stream(gray, vw, vh) {
                let last_code = LAST_DETECTED_CODE.with(|code| code.borrow().clone());

                if let Some(ref last) = last_code {
                    if last == &text {
                        let count = DETECTION_COUNT.with(|c| c.get()) + 1;
                        DETECTION_COUNT.with(|c| c.set(count));
                        if count >= REQUIRED_CONSECUTIVE_DETECTIONS {
                            invoke_on_detect(Ok(&text));
                            DETECTION_COUNT.with(|c| c.set(0));
                        }
                    } else {
                        LAST_DETECTED_CODE.with(|code| {
                            *code.borrow_mut() = Some(text.clone());
                        });
                        DETECTION_COUNT.with(|c| c.set(1));
                    }
                } else {
                    LAST_DETECTED_CODE.with(|code| {
                        *code.borrow_mut() = Some(text.clone());
                    });
                    DETECTION_COUNT.with(|c| c.set(1));
                }
            }

            if let Some(cb) = raf_cb2.borrow().as_ref() {
                window
                    .request_animation_frame(cb.as_ref().unchecked_ref())
                    .ok();
            }
        }));

        if let Some(cb) = raf_cb.borrow().as_ref() {
            window
                .request_animation_frame(cb.as_ref().unchecked_ref())
                .ok();
        }
    });

    Ok(())
}

pub fn stop_stream_scan() {
    if !STREAMING.with(|s| s.get()) {
        return;
    }

    let doc = DOCUMENT_REF.with(|doc_ref| doc_ref.borrow().clone());
    let Some(doc) = doc else {
        return;
    };

    let video_element_id = VIDEO_ELEMENT_ID.with(|id| id.borrow().clone());
    let Some(video_element_id) = video_element_id else {
        return;
    };

    if let Some(video_el) = doc
        .get_element_by_id(&video_element_id)
        .and_then(|e| e.dyn_into::<HtmlVideoElement>().ok())
        && let Some(src_obj) = video_el.src_object()
        && let Ok(stream) = src_obj.dyn_into::<MediaStream>()
    {
        stop_stream_scan_internal(&video_el, &stream);

        VIDEO_ELEMENT_ID.with(|id| {
            *id.borrow_mut() = None;
        });
    }
}

fn stop_stream_scan_internal(video: &HtmlVideoElement, stream: &MediaStream) {
    STREAMING.with(|s| s.set(false));

    LAST_DETECTED_CODE.with(|code| {
        *code.borrow_mut() = None;
    });
    DETECTION_COUNT.with(|c| c.set(0));

    RUNNING_FLAG.with(|flag| {
        if let Some(running) = flag.borrow().as_ref() {
            running.set(false);
        }
        *flag.borrow_mut() = None;
    });

    let tracks = stream.get_tracks();
    let len = tracks.length();
    for i in 0..len {
        if let Some(js_val) = tracks.get(i).dyn_ref::<MediaStreamTrack>() {
            js_val.stop();
        }
    }

    video.set_src_object(None);

    invoke_on_stop();
}
