.PHONY: run build build-release-windows build-release-linux shaders check test clippy clippy-hack fmt lint cic cicl clean

# run and compile
run:
	cargo +nightly run --example brdf

build:
	cargo +nightly build --example minimal
	cargo +nightly build --example brdf

shaders:
	ve_shader ./shaders/* -o ./assets/shaders/

build-release-windows: shaders build
	xcopy /s /y "assets\*" ".\out\assets\*"
	xcopy /s /y "target\release\examples\*" "out\"

build-release-linux: shaders build
	mkdir -p ./out/assets/
	cp -R ./assets/* ./out/assets/
	cp ./target/release/examples/* ./out/

# test and lint
check:
	cargo +nightly check --all

test:
	cargo +nightly test --all

clippy:
	cargo +nightly clippy -- -D warnings
	cargo +nightly clippy --all-targets -- -D warnings

clippy-hack:
	# hack to update files so that clippy/cargo does not use cached versions
	find -name "*.rs" -not -path "./target/*" -exec touch "{}" +
	cargo +nightly clippy --all-targets -- -D warnings

fmt:
	cargo +nightly fmt --all -- --check

lint: fmt clippy

# utility
## can i commit?
cic: test lint

## cic hack
cicl: test fmt clippy-hack

clean:
	cargo clean
