SHELL=/bin/bash

RUST_BINARY=target/x86_64-unknown-linux-gnu/debug/syscall_test
TEST_BINARY=/tmp/test_binary

# Make the path for the new interp location the same length as the original ld-linux dynamic loader
# this avoids a resize, which I can't figure out how to do properly.
#               /lib64/ld-linux-x86-64.so.2
NEW_INTERP_PATH=/tmp/new_interp_padding____
OLD_INTERP=/lib64/ld-linux-x86-64.so.2


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
	readelf -s $(RUST_BINARY)
.PHONY: ldd


clean:
	rm $(RUST_BINARY)
.PHONY: clean


test_interp_ldd:
	ldd $(TEST_BINARY)
.PHONY: test_interp_ldd

test_interp_interp:
	objdump -s -j .interp $(TEST_BINARY)
	readelf -x .interp $(TEST_BINARY)
.PHONY: test_interp_interp

test_interp_readelf_all:
	readelf -a $(TEST_BINARY)
.PHONY: test_interp_readelf_all

test_interp_build_binary:
	g++ ./test_interp/main.cpp -o $(TEST_BINARY)
.PHONY: test_interp_build_binary

test_interp_swap_interp:
	# Need our interp replacement at an absolute fixed path.
	cp $(RUST_BINARY) $(NEW_INTERP_PATH)
	# objcopy... should be the thing to use... but it results in a garbled binary.
	#objcopy --update-section .interp=/tmp/new_interp /tmp/test_binary /tmp/patched_binary
	sed -i "s|$(OLD_INTERP)|$(NEW_INTERP_PATH)|" $(TEST_BINARY)
.PHONY: test_interp_swap_interp

test_interp: build test_interp_build_binary test_interp_swap_interp
	echo "Here goes!"
	$(TEST_BINARY) arg0 arg1 arg2
.PHONY: test_interp
