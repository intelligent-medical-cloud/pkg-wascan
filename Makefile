init:
	rustup target add wasm32-unknown-unknown
	cargo install wasm-bindgen-cli

lib-upd:
	cargo update --verbose

lint:
	cargo clippy --all-targets --all-features -- -D warnings

build: lint
	@mkdir -p pkg
	cargo build --release --target wasm32-unknown-unknown
	wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/release/wascan.wasm

test: build
	@echo "Starting local server at http://localhost:8000/test"
	python3 -m http.server
	

.PHONY: init lib-upd lint build test
