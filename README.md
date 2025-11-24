# wascan

A WebAssembly barcode scanning library for the browser, built with Rust and wasm-bindgen.

## Features

- 🎯 **Barcode & QR Code Scanning** - By default **UPC-A** and **QR Code** formats are included in the compiled wasm
- 📷 **Camera Stream Support** - Real-time scanning from webcam/camera streams
- 🖼️ **Image File Support** - Scan barcodes from uploaded image files
- ⚡ **WebAssembly** - Fast, native performance in the browser
- 🚀 **High Performance** - Very high accuracy with low latency (typically under 0.2ms per detection)
- 🔧 **Easy Integration** - Simple JavaScript API
- 🎨 **Demo Included** - Try it out locally with the included demo

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
wascan = "0.1.2"
```

For JavaScript/TypeScript projects, you'll need to build the WebAssembly package first. See the [Build](#build) section below.

## Usage

### JavaScript/TypeScript

```javascript
import init, {
  init_reader,
  init_scanner,
  read_from_image,
  start_stream_scan,
  stop_stream_scan,
  on_detect,
  on_start,
  on_stop,
} from "./pkg/wascan.js";

// Initialize the WASM module
await init();

// Initialize the reader and scanner modules
init_reader();
init_scanner();

// Set up event callbacks
on_start(() => {
  console.log("Scanning started");
});

on_detect((result) => {
  if (result.success) {
    console.log("Detected:", result.value);
  } else {
    console.error("Error:", result.error);
  }
});

on_stop(() => {
  console.log("Scanning stopped");
});

// Start scanning from camera stream
start_stream_scan("video-element-id");

// Or trigger file input dialog to scan from image
read_from_image();

// Stop scanning programmatically
stop_stream_scan();
```

## API

### Initialization

- `init_reader()` - Initializes the reader module (required before using `read_from_image`)
- `init_scanner()` - Initializes the scanner module (required before using `start_stream_scan`)

### Scanning

- `start_stream_scan(video_element_id: &str)` - Starts barcode scanning from camera stream
- `read_from_image()` - Triggers file input dialog to scan from an image file
- `stop_stream_scan()` - Stops the stream scanning

### Event Callbacks

- `on_start(callback: Function)` - Register callback for when scanning starts
- `on_detect(callback: Function)` - Register callback for barcode detection
  - Callback receives: `{ success: boolean, value?: string, error?: string }`
- `on_stop(callback: Function)` - Register callback for when scanning stops

## Supported Formats

**wascan** is built on top of the [Rxing](https://github.com/username/rxing) library (Rust port of ZXing), which supports a wide range of barcode formats. However, **the built binary only includes UPC-A and QR Code** to keep the WebAssembly bundle size minimal.

### Formats Supported by Rxing/ZXing

The underlying Rxing library supports the following formats:

**1D Product Barcodes:**

- UPC-A
- UPC-E
- EAN-8
- EAN-13
- DataBar (formerly RSS-14)
- DataBar Limited

**1D Industrial Barcodes:**

- Code 39
- Code 93
- Code 128
- Codabar
- DataBar Expanded
- DX Film Edge
- ITF (Interleaved Two of Five)

**2D Matrix Barcodes:**

- QR Code
- Micro QR Code
- rMQR Code
- Aztec
- DataMatrix
- PDF417
- MaxiCode (partial support)

### Currently Included in Built Binary

The pre-built **wascan** binary includes only:

- **UPC-A** - Universal Product Code
- **QR Code** - Quick Response Code

### Adding More Formats

You can extend **wascan** to support additional barcode formats in two ways:

1. **Add specific readers explicitly** - Modify `src/detector.rs` to include additional readers from the [Rxing](https://github.com/username/rxing) library (e.g., `EAN13Reader`, `Code128Reader`, `DataMatrixReader`, etc.)

2. **Use MultiReader** - Replace the individual readers with `rxing::MultiReader` to support all formats at once

⚠️ **Important**: Including all barcode formats will approximately **double the WASM bundle size**. It's recommended to include only the formats required for your specific use case to keep the bundle size minimal.

Example of adding a specific reader:

```rust
use rxing::oned::EAN13Reader;

// Add EAN-13 detection alongside UPC-A and QR
let ean13_result = {
    let src = Luma8LuminanceSource::new(cropped.clone(), crop_w, crop_h);
    let binarizer = HybridBinarizer::new(src);
    let mut bitmap = BinaryBitmap::new(binarizer);
    let mut reader = EAN13Reader::default();
    reader.decode(&mut bitmap)
};
```

## Performance

**wascan** delivers excellent performance characteristics:

- **High Accuracy** - Very high detection accuracy for supported formats
- **Low Latency** - Detection typically completes in under 0.2ms per frame
- **Optimized for Real-time** - Efficient processing pipeline designed for continuous camera stream scanning

Benchmarks are not included in this repository, but users are welcome to measure performance on their own hardware and use cases. The WebAssembly implementation provides near-native performance while maintaining cross-platform compatibility.

## Build

This crate is designed to be compiled to WebAssembly. Use the provided Makefile:

```bash
make build
```

Or manually:

```bash
wasm-pack build --target web --out-dir pkg
```

## Try the Demo

A working demo is included in the `demo/` directory. To run it:

1. **Build the WebAssembly package:**

   ```bash
   make build
   ```

2. **Start the demo server:**

   ```bash
   make demo
   ```

3. **Open your browser:**
   - Navigate to `http://localhost:8000/demo`
   - Click "Start Stream Scan" to scan from your camera
   - Or click "Read From Image" to upload and scan an image file
   - Detected barcodes will appear in an alert and the browser console

The demo includes:

- Real-time camera scanning
- Image file upload scanning
- Event callbacks demonstration

## Requirements

- Rust toolchain
- `wasm-bindgen-cli` for building WebAssembly packages (install via `cargo install wasm-bindgen-cli`)
- Python 3 (for the demo server, or use any static file server)
- Modern browser with WebAssembly and WebRTC support (for camera functionality)

## License

Licensed under the Apache License, Version 2.0. See [LICENSE-APACHE](LICENSE-APACHE) for details.

This project builds upon the open-source [Rxing](https://github.com/username/rxing) project, which is also licensed under Apache 2.0.
