SHELL:=/bin/bash
all: backend common frontend
	pushd backend && \
	cargo build && \
	popd && \
	pushd frontend && \
	wasm-pack build --target web --out-name app && \
	rollup ./main.js --format iife --file=./pkg/bundle.js && \
	popd 

.PHONY: clean
clean:
	pushd backend && \
	cargo clean && \
	popd && \
	pushd frontend && \
	rm -rf pkg && \
	rm -rf target && \
	popd
