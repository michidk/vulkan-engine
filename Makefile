.PHONY: run check test lint cic clean

run:
	cargo +nightly run

check:
	cargo +nightly check

test:
	cargo +nightly test

lint:
	cargo +nightly fmt --all -- --check
	cargo +nightly clippy -- -D warnings

# can i commit?
cic: check test lint

clean:
	cargo clean
