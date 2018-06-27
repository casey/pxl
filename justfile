default: test

test:
	cargo test

life:
	cargo run --release --example life

shaders:
	cargo run --release --example shaders

fmt:
	cargo fmt

# clean up the feature branch named BRANCH
done BRANCH:
	git checkout {{BRANCH}} --
	git pull --rebase github master
	git checkout master
	git pull --rebase github master
	git diff --no-ext-diff --quiet --exit-code {{BRANCH}} --
	git branch -D {{BRANCH}}
