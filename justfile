default: test

test:
	cargo test

life:
	cargo run --release --example life

shaders:
	cargo run --release --example shaders

fmt:
	cargo fmt

# clean up feature branch BRANCH
done BRANCH:
	git checkout master
	git pull --rebase github master
	git diff --no-ext-diff --quiet --exit-code {{BRANCH}}
	git branch -D {{BRANCH}}
