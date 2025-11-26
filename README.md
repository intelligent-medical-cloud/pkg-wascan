# wascan

[![npm](https://img.shields.io/npm/v/wascan.svg)](https://www.npmjs.com/package/wascan)
[![crates.io](https://img.shields.io/crates/v/wascan.svg)](https://crates.io/crates/wascan)
[![license](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE-APACHE)

[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-ffdd00?style=flat&logo=buy-me-a-coffee&logoColor=black)](https://buymeacoffee.com/dave.imc)

A **production-ready, plug-and-play** WebAssembly barcode scanning library for the browser, built with Rust and wasm-bindgen. **wascan** handles all the complexity of camera access, streaming, and file input automatically - just integrate and start scanning.

## Features

- 🎯 **Barcode & QR Code Scanning** - By default only **UPC-A** and **QR Code** are included in the compiled wasm
- 📷 **Automatic Camera Handling** - Camera access and streaming handled automatically with optimal configurations applied
- 🖼️ **Built-in File Input** - File input field creation and handling managed by the library
- 🌐 **Universal Browser Support** - Works on all modern browsers and platforms (iOS, Android, Safari, Chrome, Firefox, and Edge)
- ⚡ **WebAssembly** - Fast, native performance in the browser
- 🚀 **High Performance** - Very high accuracy with low latency (typically under 0.2ms per detection)
- 🔧 **Easy Integration** - Simple JavaScript API
- 🎨 **Demo Included** - Try it out locally with the included demo

## Installation

### npm (Recommended for Web Projects)

```bash
npm install wascan
```

Or with yarn:

```bash
yarn add wascan
```

Or with pnpm:

```bash
pnpm add wascan
```

### Cargo (Rust Projects)

Add this to your `Cargo.toml`:

```toml
[dependencies]
wascan = "0.1.8"
```

For JavaScript/TypeScript projects using npm, the package is ready to use. For Rust projects or custom builds, see the [Build](#build) section below.

## Usage

### JavaScript/TypeScript

#### Basic Usage (Simple HTML/No Bundler)

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
} from "wascan";

// Initialize the WASM module (automatic path resolution)
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

#### Usage with Bundlers (Vite, Webpack, etc.)

When using bundlers like Vite or Webpack, you may need to explicitly specify the WASM file path.

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
} from "wascan";

// For Vite: Import WASM as URL
import wasmUrl from "wascan/wascan_bg.wasm?url";

// For other bundlers: Construct URL explicitly
const wasmUrl = new URL("wascan/wascan_bg.wasm", import.meta.url);

// Initialize with explicit WASM path (object format)
await init({ module_or_path: wasmUrl });

// ... rest of initialization code
```

#### Vue.js Example

```javascript
import { onMounted, onBeforeUnmount } from "vue";
import init, {
  init_reader,
  init_scanner,
  start_stream_scan,
  stop_stream_scan,
  on_detect,
  on_start,
  on_stop,
} from "wascan";

// For Vite
import wasmUrl from "wascan/wascan_bg.wasm?url";

// For other bundlers
// const wasmUrl = new URL("wascan/wascan_bg.wasm", import.meta.url);

onMounted(async () => {
  try {
    await init({ module_or_path: wasmUrl });
    init_reader();
    init_scanner();

    on_start(() => console.log("Scanning started"));
    on_detect((result) => {
      if (result.success) {
        console.log("Detected:", result.value);
      }
    });
    on_stop(() => console.log("Scanning stopped"));

    start_stream_scan("video-element-id");
  } catch (error) {
    console.error("Failed to initialize:", error);
  }
});

onBeforeUnmount(() => {
  stop_stream_scan();
});
```

## API

### Initialization

- `init(module_or_path?)` - Initializes the WASM module
  - Optional parameter: `{ module_or_path: string | URL }` - Explicit path to WASM file
  - If not provided, automatically resolves to `wascan_bg.wasm` relative to the module
  - Returns a Promise that resolves when WASM is loaded
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

**wascan** is built on top of the [Rxing](https://github.com/rxing-core/rxing) library (Rust port of ZXing), which supports a wide range of barcode formats. However, **the built binary only includes UPC-A and QR Code** to keep the WebAssembly bundle size minimal.

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

This will generate ready to use WebAssembly files in the `pkg/` directory.

## Try the Demo

A working demo is included in the `demo/` directory. To run it:

```bash
make demo
```

- Navigate to `http://localhost:8000/demo`
- Click "Start Stream Scan" to scan from your camera
- Or click "Read From Image" to upload and scan an image file
- Detected barcodes will appear in an alert and the browser console

**Note**: The demo uses the local `pkg/` directory. For npm package usage examples, see the [Usage](#usage) section above.

## Browser & Platform Support

**wascan** works seamlessly across all modern browsers and platforms:

- ✅ **Desktop**: Chrome, Firefox, Safari, Edge
- ✅ **Mobile**: iOS Safari, Chrome Android, Firefox Mobile
- ✅ **Tablets**: iPad, Android tablets
- ✅ **WebAssembly**: Required (supported by all modern browsers)

No platform-specific code needed - the library handles all browser differences automatically.

## Requirements

- Rust toolchain (for building from source)
- `wasm-bindgen-cli` and `wasm32-unknown-unknown` for building WebAssembly packages (install via `make init`)
- Python 3 (for the demo server, or use any static file server)
- Modern browser with WebAssembly and WebRTC support (for camera functionality)

## Supporting wascan?

[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-ffdd00?style=for-the-badge&logo=buy-me-a-coffee&logoColor=black)](https://buymeacoffee.com/dave.imc)

## License

Licensed under the Apache License, Version 2.0. See [LICENSE-APACHE](LICENSE-APACHE.md) for details.

This project builds upon the open-source [Rxing](https://github.com/rxing-core/rxing) project, which is also licensed under Apache 2.0.
