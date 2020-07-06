SHELL:=/bin/bash
all: backend common frontend
	rm -rf out && \
	mkdir out && \
	mkdir out/pkg && \
	pushd backend && \
	cargo build --release && \
	cp target/release/backend ../out/server && \
	popd && \
	pushd frontend && \
	wasm-pack build --target web --out-name app && \
	rollup ./main.js --format iife --file=./pkg/bundle.js && \
	cp index.html data.json Serif/* ../out && \
	cp pkg/app_bg.wasm pkg/bundle.js ../out/pkg && \
	popd  && \
	cd out && \
	./server

.PHONY: clean
clean:
	pushd backend && \
	cargo clean && \
	popd && \
	pushd frontend && \
	rm -rf pkg && \
	rm -rf target && \
	popd && \
	rm -fr out
