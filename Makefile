.PHONY: run build build-release release check test clippy fmt lint cic clean install

# run and compile
run:
	cargo +nightly run --example brdf

build:
	cargo +nightly build --example minimal
	cargo +nightly build --example brdf

build-release:
	cargo +nightly build --release --example minimal
	cargo +nightly build --release --example brdf

ifeq ($(OS),Windows_NT)
release: prepare shaders build-release
	xcopy /s /y "assets\*" ".\out\assets\*"
	xcopy /s /y "target\release\examples\*" "out\"
else
release: prepare shaders build-release
	mkdir -p ./out/assets/
	cp -R ./assets/* ./out/assets/
	cp ./target/release/examples/* ./out/
endif

# test and lint
check:
	cargo +nightly check --all --examples

test:
	cargo +nightly test --all --examples

clippy:
	cargo +nightly clippy --all --examples -- -D warnings

fmt:
	cargo +nightly fmt --all -- --check

lint: fmt clippy

# utility
## can i commit?
cic: test lint

clean:
	cargo clean

# installs binaries
install:
	cargo install --path ./ve_asset
