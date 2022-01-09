
TEST_BINARY=target/x86_64-unknown-linux-gnu/debug/syscall_test


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

.PHONY: symbols
symbols:
	readelf -s $(TEST_BINARY)
.PHONY: ldd
ldd:
	ldd $(TEST_BINARY)

interp:
	objdump -s -j .interp $(TEST_BINARY)
	readelf -x .interp $(TEST_BINARY)
.PHONY: interp

readelf_all:
	readelf -a $(TEST_BINARY)
.PHONY: readelf_all

clean:
	rm $(TEST_BINARY)
.PHONY: clean
