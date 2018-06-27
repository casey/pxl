# run all tests
default: test

# submit a pull request
pr: fmt clippy test
	@echo Checking for FIXME/TODO...
	! grep --color -En 'FIXME|TODO' src/*.rs
	@echo Checking for long lines...
	! grep --color -En '.{101}' src/*.rs
	git branch | grep '^ *master'
	git diff --exit-code
	git diff --cached --exit-code
	git push github

# run all tests
test:
	cargo test

# format rust sourcecode with rustfmt
fmt:
	cargo fmt

watch:
	cargo watch --clear --exec fmt --exec check

# everyone's favorite animate paper clip
clippy:
	cargo +nightly clippy

# run the conway's game of life example
life:
	cargo run --package pxl --release --example life

# run the custom shader example
shaders:
	cargo run --package pxl --release --example shaders

# clean up the feature branch named BRANCH
done BRANCH:
	git checkout {{BRANCH}} --
	git pull --rebase github master
	git checkout master
	git pull --rebase github master
	git diff --no-ext-diff --quiet --exit-code {{BRANCH}} --
	git branch -D {{BRANCH}}
