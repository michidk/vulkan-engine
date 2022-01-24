.PHONY: run build build-release build-shipping release check test clippy fmt lint cic clean install

# run and compile
run:
	cargo run --example brdf

build:
	cargo build -p vulkan_engine --examples

build-release:
	cargo build --release -p vulkan_engine --examples

build-shipping:
	cargo build --profile shipping -p vulkan_engine --examples

ifeq ($(OS),Windows_NT)
package: build-shipping
	xcopy "assets" "out\assets" /i /s /y
	xcopy "target\shipping\examples\*.exe" "out" /i /s /y
else
package: build-shipping
	mkdir -p ./out/assets/
	cp -R ./assets/* ./out/assets/
	cp ./target/shipping/examples/* ./out/
endif

# test and lint
check:
	cargo check --workspace --all-targets

test:
	cargo test --workspace --all-targets

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

fmt:
	cargo fmt --all -- --check

lint: fmt clippy

# utility
## can i commit?
cic: test lint

ifeq ($(OS),Windows_NT)
clean:
	cargo clean
	rd /s /q "./assets/shaders"
else
clean:
	cargo clean
	rm -r ./assets/shaders
endif

# installs binaries
install:
	cargo install --path ./ve_asset
