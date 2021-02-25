.PHONY: run build build-release shaders build-release-windows build-release-linux check test clippy clippy-hack fmt lint cic cicl prepare-release-windows prepare-release-linux clean

# run and compile
run:
	cargo +nightly run --example brdf

build:
	cargo +nightly build --example minimal
	cargo +nightly build --example brdf

build-release:
	cargo +nightly build --example minimal
	cargo +nightly build --example brdf

shaders:
	ve_shader "./shaders/*" -o ./assets/shaders/

# build on windows for windows with make for windows
build-release-windows: prepare-release-windows shaders build-release
	xcopy /s /y "assets\*" ".\out\assets\*"
	xcopy /s /y "target\release\examples\*" "out\"

# build on windows for windows with mingw-make
build-release-mingw: prepare-release-linux shaders build-release
	mkdir -p ./out/assets/
	cp -R ./assets/* ./out/assets/
	cp ./target/release/examples/* ./out/

# build on linux for linux with make for linux
build-release-linux: prepare-release-linux shaders build-release
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

prepare-release-windows:
	mkdir "assets\shaders"

prepare-release-linux:
	mkdir -p ./assets/shaders

clean:
	cargo clean
