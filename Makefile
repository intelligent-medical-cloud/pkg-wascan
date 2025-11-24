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
	@echo "Note: Ensure pkg/package.json version matches Cargo.toml version before publishing to npm"

demo: build
	@echo "Starting local server at http://localhost:8000/demo"
	python3 -m http.server

.PHONY: init lib-upd lint build demo

publish-npm: build
	@cd pkg && npm publish

.PHONY: publish-npm
