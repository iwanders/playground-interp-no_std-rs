## On rebuilding the core library
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

-> Further analysis shows that enabling optimisations, even of `opt-level = 1` cause the segfault to happen. (7be297d339bd4e44f570f5362cc75c4138f1c21b)

## Using a stage1 toolchain.

So in an attempt to get proper debug symbols for the `core` module, I followed [this](https://rustc-dev-guide.rust-lang.org/building/how-to-build-and-run.html#building-the-compiler) set `debug = true` in the `[rust]` block of the `config.toml` file, and tried again.

```
./x.py build -i library/core
```

```
Program received signal SIGSEGV, Segmentation fault.
0x0000555555559315 in core::num::flt2dec::to_shortest_str () at library/core/src/num/flt2dec/mod.rs:364
364	    let (negative, full_decoded) = decode(v);
(gdb) bt
#0  0x0000555555559315 in core::num::flt2dec::to_shortest_str () at library/core/src/num/flt2dec/mod.rs:364
#1  0x0000555555558fe2 in core::fmt::float::float_to_decimal_common_shortest () at library/core/src/fmt/float.rs:66
#2  0x000055555555705e in core::fmt::write () at library/core/src/fmt/mod.rs:1168
#3  0x0000555555556744 in syscall_test::_start () at src/main.rs:31
(gdb) frame 0
#0  0x0000555555559315 in core::num::flt2dec::to_shortest_str () at library/core/src/num/flt2dec/mod.rs:364
364	    let (negative, full_decoded) = decode(v);
(gdb) info locals
No locals.
```

So, still no proper debug information (we would at least get 'optimised out')?

Ah, from the config;
```
# Debuginfo level for most of Rust code, corresponds to the `-C debuginfo=N` option of `rustc`.
# `0` - no debug info
# `1` - line tables only - sufficient to generate backtraces that include line
#       information and inlined functions, set breakpoints at source code
#       locations, and step through execution in a debugger.
# `2` - full debug info with variable and type information
```

That makes sense, and is unfortunate, some warning about gigabytes of debug symbols... And;

> Can be overridden for specific subsets of Rust code (rustc, std or tools).

Ok, recompiling with that set to `2` for the std.