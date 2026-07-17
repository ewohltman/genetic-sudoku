.DEFAULT_GOAL := default

.PHONY: default
default: build

.PHONY: clean
clean:
	cargo clean

.PHONY: fmt
fmt:
	cargo fmt --all --check

.PHONY: clippy
clippy:
	cargo clippy --all-targets -- -D warnings

.PHONY: test
test:
	cargo test

.PHONY: bench
bench:
	cargo bench

.PHONY: bench-check
bench-check:
	cargo bench --no-run

.PHONY: build
build:
	cargo build --release

.PHONY: install
install:
	cargo install --path .

.PHONY: ci
ci: fmt clippy test bench-check build

.PHONY: vhs
vhs:
	vhs demo/demo.tape
