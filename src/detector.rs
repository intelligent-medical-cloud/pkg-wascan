use std::io::Cursor;

use image::{
    GrayImage, ImageReader,
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
                invoke_on_stop();

                return;
            }

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

pub fn detect_from_stream(gray_data: Vec<u8>, width: u32, height: u32) -> Result<String, Error> {
    if width < MIN_IMAGE_DIMENSION || height < MIN_IMAGE_DIMENSION {
        return Err(Error::NotDetected);
    }

    let resized_w = width / RESIZE_FACTOR;
    let resized_h = height / RESIZE_FACTOR;
    let image = match GrayImage::from_raw(width, height, gray_data) {
        Some(img) => img,
        None => return Err(Error::NotDetected),
    };
    let resized_image = resize(&image, resized_w, resized_h, FilterType::Lanczos3);
    let resized_data = resized_image.into_raw();

    // Try UPC-A first
    let upca_result = {
        let src = Luma8LuminanceSource::new(resized_data.clone(), resized_w, resized_h);
        let binarizer = HybridBinarizer::new(src);
        let mut bitmap = BinaryBitmap::new(binarizer);
        let mut reader = UPCAReader::default();
        reader.decode(&mut bitmap)
    };

    if let Ok(res) = upca_result {
        return Ok(res.getText().to_string());
    }

    // Try QR code
    let qr_result = {
        let src = Luma8LuminanceSource::new(resized_data, resized_w, resized_h);
        let binarizer = HybridBinarizer::new(src);
        let mut bitmap = BinaryBitmap::new(binarizer);
        let mut reader = QRCodeReader::new();
        reader.decode(&mut bitmap)
    };

    match qr_result {
        Ok(res) => Ok(res.getText().to_string()),
        Err(_) => Err(Error::NotDetected),
    }
}
