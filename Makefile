.DEFAULT_GOAL := default

.PHONY: default
default: target/release/genetic-sudoku

.PHONY: clean
clean:
	rm -rf target

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: clippy
clippy: fmt
	cargo clippy

.PHONY: test
test: fmt
	cargo test

.PHONY: bench
bench: fmt
	cargo bench

target/release/genetic-sudoku:
	cargo build --release
