.PHONY: all

run:
	cargo run +nightly

check:
	cargo check +nightly

test:
	cargo test +nightly

lint:
	cargo fmt +nightly --all -- --check

clean:
	cargo clean

