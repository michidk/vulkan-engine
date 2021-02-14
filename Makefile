.PHONY: run check test clippy clippy-hack fmt lint cic cicl clean

run:
	cargo +nightly run

check:
	cargo +nightly check --all

test:
	cargo +nightly test --all

clippy:
	cargo +nightly clippy -- -D warnings

clippy-hack:
	# hack to update files so that clippy/cargo does not use cached versions
	find -name "*.rs" -not -path "./target/*" -exec touch "{}" +
	cargo +nightly clippy -- -D warnings

fmt:
	cargo +nightly fmt --all -- --check

lint: fmt clippy

# can i commit?
cic: test fmt clippy
	cargo +nightly clippy -- -D warnings

# cic hack
cicl: test fmt clippy-hack

clean:
	cargo clean
