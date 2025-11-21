use std::io::Cursor;

use image::{
    ImageReader,
    imageops::{FilterType, resize},
};
use js_sys::Uint8Array;
use rxing::{
    BinaryBitmap, Luma8LuminanceSource, Reader, common::HybridBinarizer, oned::UPCAReader,
    qrcode::QRCodeReader,
};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{Event, File, FileReader};

use crate::{
    error::Error,
    event::{invoke_on_detect, invoke_on_stop},
};

const MIN_IMAGE_DIMENSION: u32 = 10;

const RESIZE_FACTOR: u32 = 2;

pub fn detect_from_image(file: File) {
    let Ok(reader) = FileReader::new() else {
        invoke_on_detect(Err(&Error::Internal));
        invoke_on_stop();

        return;
    };

    let onload = {
        let reader_ref = reader.clone();
        Closure::wrap(Box::new(move |_evt: Event| {
            let Ok(js_val) = reader_ref.result() else {
                invoke_on_detect(Err(&Error::Internal));
                invoke_on_stop();

                return;
            };

            let array = Uint8Array::new(&js_val);
            let mut input_bytes = vec![0u8; array.length() as usize];
            array.copy_to(&mut input_bytes);

            let dyn_image = match ImageReader::new(Cursor::new(&input_bytes)).with_guessed_format()
            {
                Ok(rdr) => match rdr.decode() {
                    Ok(img) => img,
                    Err(_) => {
                        invoke_on_detect(Err(&Error::Internal));
                        invoke_on_stop();

                        return;
                    }
                },
                Err(_) => {
                    invoke_on_detect(Err(&Error::Internal));
                    invoke_on_stop();

                    return;
                }
            };

            let gray = dyn_image.to_luma8();
            // TODO: Make image processing logic configurable
            let w = gray.width() / RESIZE_FACTOR;
            let h = gray.height() / RESIZE_FACTOR;
            if w < MIN_IMAGE_DIMENSION || h < MIN_IMAGE_DIMENSION {
                invoke_on_detect(Err(&Error::NotDetected));
                invoke_on_stop();

                return;
            }
            let gray_resized = resize(&gray, w, h, FilterType::Lanczos3);
            let gray_data = gray_resized.into_raw();

            // Try UPC-A
            let upca_result = {
                let src = Luma8LuminanceSource::new(gray_data.clone(), w, h);
                let binarizer = HybridBinarizer::new(src);
                let mut bitmap = BinaryBitmap::new(binarizer);
                let mut reader = UPCAReader::default();
                reader.decode(&mut bitmap)
            };

            if let Ok(res) = upca_result {
                invoke_on_detect(Ok(res.getText()));
            } else {
                // Try QR
                let qr_result = {
                    let src = Luma8LuminanceSource::new(gray_data, w, h);
                    let binarizer = HybridBinarizer::new(src);
                    let mut bitmap = BinaryBitmap::new(binarizer);
                    let mut reader = QRCodeReader::new();
                    reader.decode(&mut bitmap)
                };

                match qr_result {
                    Ok(res) => invoke_on_detect(Ok(res.getText())),
                    Err(_) => invoke_on_detect(Err(&Error::NotDetected)),
                }
            }

            invoke_on_stop();
        }) as Box<dyn FnMut(_)>)
    };

    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();

    if reader.read_as_array_buffer(&file).is_err() {
        invoke_on_detect(Err(&Error::Internal));
        invoke_on_stop();
    }
}
