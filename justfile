default: test

test:
	cargo test
	cd examples/life && cargo test

life:
	cd examples/life && cargo run --release

shaders:
	cd examples/shaders && cargo run --release

fmt:
	cargo fmt
