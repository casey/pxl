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

# clean up feature branch BRANCH
done BRANCH:
	git checkout master
	git pull --rebase github master
	git diff --no-ext-diff --quiet --exit-code {{BRANCH}}
	git branch -D {{BRANCH}}
