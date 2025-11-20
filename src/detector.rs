use std::io::Cursor;

use image::ImageReader;
use js_sys::Uint8Array;
use rxing::{
    BinaryBitmap, Luma8LuminanceSource, Reader, common::HybridBinarizer, oned::UPCAReader,
};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{Event, File, FileReader};

use crate::{
    error::Error,
    event::{invoke_on_detect, invoke_on_stop},
};

pub fn detect_from_image(file: File) {
    let mime = file.type_();
    if !mime.starts_with("image/") {
        invoke_on_detect(Err(&Error::NotImageFile));
        invoke_on_stop();

        return;
    }

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
            // TODO: img processing flexible logic
            let w = gray.width();
            let h = gray.height();
            if w < 10 || h < 10 {
                invoke_on_detect(Err(&Error::NotDetected));
                invoke_on_stop();

                return;
            }

            let src = Luma8LuminanceSource::new(gray.as_raw().to_vec(), w, h);
            let binarizer = HybridBinarizer::new(src);
            let mut bitmap = BinaryBitmap::new(binarizer);
            let mut upca_reader = UPCAReader::default();

            match upca_reader.decode(&mut bitmap) {
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
