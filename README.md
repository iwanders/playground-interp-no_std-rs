# Playground interp no_std

Odd but descriptive repo name. This repo isn't intended to _really_ do anything, it is just me
playing around trying to make a completely stand-alone binary in Rust. Without relying on libc, the
dynamic linker and basically all other conveniences we take for granted normally. So manually
implementing the interaction between userspace and the kernel to print to stdout.

My stretch goal was making something that can actually act as an `.interp` target for a dynamically
linked binary, which does work, the built target from `make build` can be used as a substitute for
`/lib64/ld-linux-x86-64.so.2` which is usually in the `.interp` section of dynamically linked
binaries. When this is done, obviously the original binary doesn't run, instead the code from
`main.rs` runs, after which it exits.

Do not use this for anything, it's probably riddled with bugs.

## On all the flags
- So we need nightly in 'rust-toolchain.toml' to get access to the new-style `asm!` macro.
- `-C link-arg=-nostartfile` to prevent linking with libc.
- `-C link-arg=-static` to tell the compiler to make a static binary.
- `-C relocation-model=pic` to ensure the code is position independent, which is necessary when using as an interpreter.
- `panic = "abort"` to avoid requirements on `eh_personality`.


## Notes
The `core` library _must_ be rebuilt, I'm not certain why, but if we don't and we format a 
floating point number we get a segfault from somewhere in the float formatter;
```
==5142== Process terminating with default action of signal 11 (SIGSEGV)
==5142==  General Protection Fault
==5142==    at 0x10FDAC: format_shortest (grisu.rs:467)
==5142==    by 0x10FDAC: call_mut<fn(&core::num::flt2dec::decoder::Decoded, &mut [core::mem::maybe_uninit::MaybeUninit<u8>]) -> (&[u8], i16),(&core::num::flt2dec::decoder::Decoded, &mut [core::mem::maybe_uninit::MaybeUninit<u8>])> (function.rs:150)
==5142==    by 0x10FDAC: to_shortest_str<f64,fn(&core::num::flt2dec::decoder::Decoded, &mut [core::mem::maybe_uninit::MaybeUninit<u8>]) -> (&[u8], i16)> (mod.rs:497)
==5142==    by 0x10FDAC: core::fmt::float::float_to_decimal_common_shortest (float.rs:45)
==5142==    by 0x10A836: <&T as core::fmt::Debug>::fmt (mod.rs:2012)
==5142==    by 0x10F5DE: {{closure}} (builders.rs:344)
==5142==    by 0x10F5DE: and_then<(),core::fmt::Error,(),closure-0> (result.rs:704)
==5142==    by 0x10F5DE: core::fmt::builders::DebugTuple::field (builders.rs:331)
==5142==    by 0x10A763: <core::option::Option<T> as core::fmt::Debug>::fmt (option.rs:158)
==5142==    by 0x11043E: core::fmt::write (mod.rs:1094)
==5142==    by 0x10B2D6: _start (main.rs:286)
```

So basically, we need to rebuild the core library.
```
cargo b -Z build-std=core --target x86_64-unknown-linux-gnu
```

The Makefile provides convenience helpers for this that allow using `make r` to run the above command.

