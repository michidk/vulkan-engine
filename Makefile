.PHONY: all

run:
	cargo run +nightly

check:
	cargo check +nightly

test:
	cargo test +nightly

clean:
	cargo clean
