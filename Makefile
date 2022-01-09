
TEST_BINARY=target/debug/syscall_test


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


# If we strip interp, we can't start anymore... probably missing some magic I don't understand yet.
remove_interp:
	objcopy --remove-section .interp $(TEST_BINARY)
.PHONY: remove_interp
