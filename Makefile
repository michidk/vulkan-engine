.PHONY: run build build-release build-production release check test clippy fmt lint cic clean install

# run and compile
run:
	cargo run --example brdf

build:
	cargo build -p vulkan_engine --examples

build-release:
	cargo build --release -p vulkan_engine --examples

build-production:
	cargo build --profile production -p vulkan_engine --examples

ifeq ($(OS),Windows_NT)
package: build-production
	powershell Copy-Item -Path "assets" -Destination "out\assets" -Recurse
	powershell Copy-Item -Path "target\production\examples\*" -Destination "out" -Include "*.exe"
else
package: build-production
	mkdir -p ./out/assets/
	cp -R ./assets/* ./out/assets/
	for f in $(shell ls crates/engine/examples); do cp target/production/examples/$$f out; done
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
