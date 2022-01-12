run:
	cargo run --target x86_64-unknown-linux-gnu
.PHONY: run
r: run
.PHONY: r


build:
	cargo build --target x86_64-unknown-linux-gnu
.PHONY: build

b: build
.PHONY: b

