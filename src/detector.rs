use std::io::Cursor;

use image::{
    GrayImage, ImageReader,
    imageops::{FilterType, crop, resize},
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

const MIN_IMAGE_DIMENSION: u32 = 60;
const OPTIMAL_IMAGE_DIMENSION: u32 = 1200;
const IMAGE_CROP_FACTOR: u32 = 2;
const STREAM_CROP_FACTOR: u32 = 2;

fn prepare_image_data(image: &GrayImage, width: u32, height: u32) -> (Vec<u8>, u32, u32) {
    if width > OPTIMAL_IMAGE_DIMENSION || height > OPTIMAL_IMAGE_DIMENSION {
        let ratio = width as f64 / height as f64;
        let new_w = if ratio > 1.0 {
            OPTIMAL_IMAGE_DIMENSION
        } else {
            (OPTIMAL_IMAGE_DIMENSION as f64 * ratio) as u32
        };
        let new_h = if ratio > 1.0 {
            (OPTIMAL_IMAGE_DIMENSION as f64 / ratio) as u32
        } else {
            OPTIMAL_IMAGE_DIMENSION
        };
        let resized = resize(image, new_w, new_h, FilterType::Lanczos3);
        (resized.into_raw(), new_w, new_h)
    } else {
        (image.clone().into_raw(), width, height)
    }
}

fn detect_barcode(gray_data: Vec<u8>, width: u32, height: u32) -> Result<String, Error> {
    // Try UPC-A first
    let upca_result = {
        let src = Luma8LuminanceSource::new(gray_data.clone(), width, height);
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
        let src = Luma8LuminanceSource::new(gray_data, width, height);
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

            if dyn_image.width() < OPTIMAL_IMAGE_DIMENSION
                || dyn_image.height() < OPTIMAL_IMAGE_DIMENSION
            {
                invoke_on_detect(Err(&Error::NotDetected));
                invoke_on_stop();

                return;
            }

            let gray = dyn_image.to_luma8();
            let full_width = gray.width();
            let full_height = gray.height();

            let crop_w = full_width / IMAGE_CROP_FACTOR;
            let crop_h = full_height / IMAGE_CROP_FACTOR;
            let crop_x = (full_width - crop_w) / 2;
            let crop_y = (full_height - crop_h) / 2;
            let mut cropped_gray = gray.clone();
            let cropped = crop(&mut cropped_gray, crop_x, crop_y, crop_w, crop_h).to_image();

            let (gray_data, w, h) = prepare_image_data(&cropped, cropped.width(), cropped.height());

            let result = match detect_barcode(gray_data, w, h) {
                Ok(text) => {
                    invoke_on_detect(Ok(text.as_str()));
                    invoke_on_stop();
                    return;
                }
                Err(_) => {
                    let (full_gray_data, full_w, full_h) =
                        prepare_image_data(&gray, full_width, full_height);
                    detect_barcode(full_gray_data, full_w, full_h)
                }
            };

            match result {
                Ok(text) => invoke_on_detect(Ok(text.as_str())),
                Err(e) => invoke_on_detect(Err(&e)),
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

    let crop_w = width / STREAM_CROP_FACTOR;
    let crop_h = height / STREAM_CROP_FACTOR;
    let crop_x = (width - crop_w) / 2;
    let crop_y = (height - crop_h) / 2;

    let mut cropped = vec![0u8; (crop_w * crop_h) as usize];
    for y in 0..crop_h {
        for x in 0..crop_w {
            let src_idx = ((crop_y + y) * width + (crop_x + x)) as usize;
            let dst_idx = (y * crop_w + x) as usize;
            cropped[dst_idx] = gray_data[src_idx];
        }
    }

    detect_barcode(cropped, crop_w, crop_h)
}
