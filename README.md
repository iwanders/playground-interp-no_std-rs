# Playground interp no_std

Odd but descriptive repo name. This repo isn't intended to _really_ do anything, it is just me
playing around trying to make a completely stand-alone binary in Rust. Without relying on libc, the
dynamic linker and basically all other conveniences we take for granted normally. So manually
implementing the interaction systemcall between userspace and the kernel to print to stdout.

The binary built in this crate can act as an `.interp` target for a dynamically
linked binary. We can do so by just replacing the  `/lib64/ld-linux-x86-64.so.2` written in the
`.interp` section. Or just run `make test_interp`, which builds a binary and does the appropriate
substitution.

When this is done, obviously the original binary doesn't run, instead the code from `main.rs` runs
first, before it dispatches to the binary it is acting as an interpreter for. This will probably
result in a segfault as soon as the test binary calls a function that is from glibc, beause the
actual dynamic linking step is not done.

Do not use this for anything, it's probably riddled with bugs.

## On all the flags
- So we need nightly in 'rust-toolchain.toml' to get access to the new-style `asm!` macro.
- `-C link-arg=-nostartfile` to prevent linking with libc.
- `-C link-arg=-static` to tell the compiler to make a static binary.
- `-C relocation-model=pic` to ensure the code is position independent, which is necessary when using as an interpreter.
- `panic = "abort"` to avoid requirements on `eh_personality`.


## Notes

The target must be passed as well, otherwise the `-nostartfiles` gets passed to buildscripts (of any dependencies), which prevents them from running succesfully.
```
cargo b --target x86_64-unknown-linux-gnu
```
