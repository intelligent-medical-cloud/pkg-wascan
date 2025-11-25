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

demo: build
	@echo "Starting local server at http://localhost:8000/demo"
	python3 -m http.server

.PHONY: init lib-upd lint build demo

publish-npm: build
	@cp README.md pkg/README.md
	@cp package.json pkg/package.json
	@cd pkg && npm publish

publish-cargo: build
	cargo package
	cargo package --list
	cargo doc --no-deps
	cargo publish

.PHONY: publish-npm publish-cargo
