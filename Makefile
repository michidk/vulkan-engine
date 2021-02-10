.PHONY: run check test clippy fmt lint cic clean

run:
	cargo +nightly run

check:
	cargo +nightly check
	cargo +nightly check -p math

test:
	cargo +nightly test
	cargo +nightly test -p math

clippy:
	# hack to update files so that clippy/cargo does not use cached versions
	find -name "*.rs" -not -path "./target/*" -exec touch "{}" +
	cargo +nightly clippy -- -D warnings

fmt:
	cargo +nightly fmt --all -- --check
	cargo +nightly fmt -p math --all -- --check

lint: fmt clippy

# can i commit?
cic: check test lint

clean:
	cargo clean
