SHELL=/bin/bash

RUST_BINARY=target/x86_64-unknown-linux-gnu/debug/examples/dynamic_linker
TEST_BINARY=/tmp/test_binary

# Make the path for the new interp location the same length as the original ld-linux dynamic loader
# this avoids a resize, which I can't figure out how to do properly.
#               /lib64/ld-linux-x86-64.so.2
NEW_INTERP_PATH=/tmp/new_interp_padding____
OLD_INTERP=/lib64/ld-linux-x86-64.so.2


# We must pass --target to ensure the -nostdlibs doesnt get passed to buildscripts.
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


clean:
	rm $(RUST_BINARY)
.PHONY: clean



run_bdl: test_binary_minimal
	cargo run --example dynamic_linker -Z build-std=core --target x86_64-unknown-linux-gnu -- /tmp/minimal
.PHONY: run_bdl
rbdl: run_bdl
.PHONY: rbdl

build_dl:
	cargo build --example dynamic_linker -Z build-std=core --target x86_64-unknown-linux-gnu
.PHONY: build_dl

bdl: build_dl
.PHONY: bdl


test_binary_minimal:
	# -DUSE_LIB=1
	g++ -g -fPIC -shared ./test_interp/minimal_lib.cpp -o /tmp/libminimal_lib.so
	g++ -g -nostdlib -fPIC ./test_interp/minimal.cpp -L/tmp/ -l minimal_lib -o $(TEST_BINARY)
.PHONY: test_binary_minimal

# This binary is really complex... relying on printf and lots more...
test_interp_build_binary:
	g++ ./test_interp/main.cpp -o $(TEST_BINARY)
	cp $(TEST_BINARY) /tmp/orig_test_binary
.PHONY: test_interp_build_binary

test_interp_swap_interp:
	# Need our interp replacement at an absolute fixed path.
	cp $(RUST_BINARY) $(NEW_INTERP_PATH)
	# objcopy... should be the thing to use... but it results in a garbled binary.
	#objcopy --update-section .interp=/tmp/new_interp /tmp/test_binary /tmp/patched_binary
	sed -i "s|$(OLD_INTERP)|$(NEW_INTERP_PATH)|" $(TEST_BINARY)
.PHONY: test_interp_swap_interp

test_interp: build_dl test_binary_minimal test_interp_swap_interp
	echo "Here goes!"
	$(TEST_BINARY) arg0 arg1 arg2
.PHONY: test_interp
