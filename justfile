default: test

test:
	cargo test
	cd examples/life && cargo test

life:
	cd examples/life && cargo run --release

fmt:
	cargo fmt
