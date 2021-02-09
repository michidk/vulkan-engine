.PHONY: run check test lint cic clean

run:
	cargo +nightly run

check:
	cargo +nightly check
	cargo +nightly check -p math

test:
	cargo +nightly test
	cargo +nightly test -p math

lint:
	cargo +nightly fmt --all -- --check
	cargo +nightly clippy -- -D warnings
	cargo +nightly fmt -p math --all -- --check
	cargo +nightly clippy -p math -- -D warnings

# can i commit?
cic: check test lint

clean:
	cargo clean
