.PHONY: run build build-release release check test clippy fmt lint cic clean install

# run and compile
run:
	cargo run --example brdf

build:
	cargo build --example minimal
	cargo build --example brdf

build-release:
	cargo build --release --example minimal
	cargo build --release --example brdf

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
	cargo check --all --examples

test:
	cargo test --all --examples

clippy:
	cargo clippy --all --examples -- -D warnings

fmt:
	cargo fmt --all -- --check

lint: fmt clippy

# utility
## can i commit?
cic: test lint

clean:
	cargo clean

# installs binaries
install:
	cargo install --path ./ve_asset
