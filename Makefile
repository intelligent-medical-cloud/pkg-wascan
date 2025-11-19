init:
	rustup target add wasm32-unknown-unknown
	cargo install wasm-bindgen-cli

lib-upd:
	cargo update --verbose

build:
	@mkdir -p pkg
	cargo build --release --target wasm32-unknown-unknown
	wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/release/wascan.wasm

.PHONY: init lib-upd build
