.PHONY: all

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

canICommit:
	make check
	make test
	make lint

clean:
	cargo clean
