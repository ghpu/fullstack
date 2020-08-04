SHELL:=/bin/bash
all: backend common frontend
	rm -rf out && \
	mkdir out && \
	mkdir out/pkg && \
	pushd backend && \
	#cargo build --release --target x86_64-pc-windows-gnu && \
	cargo build --release && \
	cp target/release/backend ../out/server && \
	popd && \
	pushd frontend && \
	wasm-pack build --target web --out-name app && \
	rollup ./main.js --output.format iife --file=./pkg/bundle.js && \
	cp index.html data.json Serif/* ../out && \
	cp pkg/app_bg.wasm pkg/bundle.js ../out/pkg

run: all
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
