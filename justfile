default: test

test:
	cargo test

life:
	cd examples/life && cargo run --release

fmt:
	cargo fmt
