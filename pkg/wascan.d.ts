/* tslint:disable */
/* eslint-disable */
/**
 * WASM entry point
 */
export function main_js(): void;
/**
 * Triggers the file input dialog to read an image file.
 */
export function read_from_image(): void;
/**
 * Initializes the reader module. Must be called before using `read_from_image`.
 */
export function init_reader(): void;
/**
 * Starts the stream-based barcode scanning from the camera.
 *
 * ## Arguments
 * * `video_element_id` - The ID of the video element in the DOM where the stream will be displayed
 */
export function start_stream_scan(video_element_id: string): void;
/**
 * Initializes the scanner module. Must be called before using `start_stream_scan`.
 */
export function init_scanner(): void;
/**
 * Stops the stream scanning programmatically.
 */
export function stop_stream_scan(): void;
export function error_codes(): any;
/**
 * Registers a callback function to be called when scanning stops.
 */
export function on_stop(cb: Function): void;
/**
 * Registers a callback function to be called when scanning starts.
 */
export function on_start(cb: Function): void;
/**
 * Registers a callback function to be called when a barcode is detected.
 *
 * The callback receives an object with:
 * - `success: boolean` - true if detection succeeded, false otherwise
 * - `value?: string` - the detected barcode (only present if success is true)
 * - `error?: string` - the error code (only present if success is false)
 */
export function on_detect(cb: Function): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly init_reader: () => [number, number];
  readonly init_scanner: () => [number, number];
  readonly main_js: () => void;
  readonly read_from_image: () => [number, number];
  readonly start_stream_scan: (a: number, b: number) => [number, number];
  readonly stop_stream_scan: () => void;
  readonly error_codes: () => any;
  readonly on_detect: (a: any) => void;
  readonly on_start: (a: any) => void;
  readonly on_stop: (a: any) => void;
  readonly wasm_bindgen__convert__closures_____invoke__had67db21a2959b7b: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__hd1614e8a7e4ac567: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__hb94288fd6ffe286a: (a: number, b: number, c: number) => void;
  readonly wasm_bindgen__closure__destroy__h28ae10224d8baff5: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h66c325097dcb36e0: (a: number, b: number, c: any) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
