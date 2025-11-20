use std::io::Cursor;

use image::ImageReader;
use js_sys::Uint8Array;
use rxing::{
    BinaryBitmap, Luma8LuminanceSource, Reader, common::HybridBinarizer, oned::UPCAReader,
};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{File, FileReader};

use crate::{
    error::Error,
    event::{invoke_on_detect, invoke_on_stop},
};

pub fn detect_from_image(file: File) {
    // Validate MIME early (defensive; reader already filters)
    let mime = file.type_();
    if !mime.starts_with("image/") {
        invoke_on_detect(Err(&Error::NotImageFile));
        return;
    }

    let Ok(reader) = FileReader::new() else {
        invoke_on_detect(Err(&Error::Internal));
        return;
    };

    // onload closure performs decode & detection
    let onload = {
        let reader_ref = reader.clone();
        Closure::wrap(Box::new(move |_evt: web_sys::Event| {
            let js_val = match reader_ref.result() {
                Ok(v) => v,
                Err(_) => {
                    invoke_on_detect(Err(&Error::Internal));
                    return;
                }
            };
            let array = Uint8Array::new(&js_val);
            let mut input_bytes = vec![0u8; array.length() as usize];
            array.copy_to(&mut input_bytes);

            // Decode image safely
            let dyn_image = match ImageReader::new(Cursor::new(&input_bytes)).with_guessed_format()
            {
                Ok(rdr) => match rdr.decode() {
                    Ok(img) => img,
                    Err(_) => {
                        invoke_on_detect(Err(&Error::DecodeFailed));
                        return;
                    }
                },
                Err(_) => {
                    invoke_on_detect(Err(&Error::DecodeFailed));
                    return;
                }
            };

            let gray = dyn_image.to_luma8();
            let w = gray.width();
            let h = gray.height();
            if w < 3 || h < 3 {
                invoke_on_detect(Err(&Error::ImageTooSmall));
                return;
            }

            let src = Luma8LuminanceSource::new(gray.as_raw().to_vec(), w, h);
            let binarizer = HybridBinarizer::new(src);
            let mut bitmap = BinaryBitmap::new(binarizer);
            let mut upca_reader = UPCAReader::default();

            match upca_reader.decode(&mut bitmap) {
                Ok(res) => invoke_on_detect(Ok(res.getText())),
                Err(_) => invoke_on_detect(Err(&Error::DecodeFailed)),
            }

            // End of detection lifecycle
            invoke_on_stop();
        }) as Box<dyn FnMut(_)>)
    };

    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget(); // Leak closure to keep callback alive

    // Kick off async read
    if reader.read_as_array_buffer(&file).is_err() {
        invoke_on_detect(Err(&Error::Internal));
        invoke_on_stop();
    }
}
