run:
	cargo run -Z build-std=core --target x86_64-unknown-linux-gnu
.PHONY: run
r: run
.PHONY: r


build:
	cargo build -Z build-std=core --target x86_64-unknown-linux-gnu
.PHONY: build

b: build
.PHONY: b

