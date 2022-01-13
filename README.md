## Initial discovery

> The `core` library _must_ be rebuilt, I'm not certain why, but if we don't and we format a 
floating point number we get a segfault from somewhere in the float formatter;

From valgrind:
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

-> Further analysis shows that enabling optimisations, even of `opt-level = 1` cause the segfault to happen. (7be297d339bd4e44f570f5362cc75c4138f1c21b), even with nightly compiler.

---

Using a stage1 toolchain.

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

Well, that failed... 
```
ivor@eagle:~/Documents/Code/rust/rust$ ./x.py build -i library/std
<snip>
   Compiling object v0.26.2
thread 'rustc' panicked at 'no entry found for key', compiler/rustc_metadata/src/rmeta/decoder/cstore_impl.rs:492:9
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

error: internal compiler error: unexpected panic

note: the compiler unexpectedly panicked. this is a bug.

note: we would appreciate a bug report: https://github.com/rust-lang/rust/issues/new?labels=C-bug%2C+I-ICE%2C+T-compiler&template=ice.md

note: rustc 1.58.0-beta.1 (426b94d7d 2021-11-29) running on x86_64-unknown-linux-gnu

note: compiler flags: -Z symbol-mangling-version=v0 -Z macro-backtrace -Z tls-model=initial-exec -Z unstable-options -Z binary-dep-depinfo -Z force-unstable-if-unmarked -C opt-level=3 -C embed-bitcode=no -C debuginfo=1 -C debug-assertions=on -C incremental -C link-args=-Wl,-z,origin -C link-args=-Wl,-rpath,$ORIGIN/../lib -C prefer-dynamic -C llvm-args=-import-instr-limit=10 --crate-type lib

note: some of the compiler flags provided by cargo are hidden

query stack during panic:
#0 [predicates_of] computing predicates of `json::FormatShim`
#1 [param_env] computing normalized predicates of `json::FormatShim`
end of query stack
error: could not compile `rustc_serialize`
warning: build failed, waiting for other jobs to finish...
error: build failed
Build completed unsuccessfully in 0:01:11
ivor@eagle:~/Documents/Code/rust/rust$
```
Looks similar to [this issue](https://github.com/rust-lang/rust/issues/91767)...  No idea why it says `rustc 1.58.0-beta.1 (426b94d7d 2021-11-29)` though... 

Lets try that `profile = "library"`...  ok that seems to have made a stage0 compiler...

```(gdb) bt
#0  0x0000555555559d81 in core::num::flt2dec::strategy::grisu::format_shortest () at library/core/src/num/flt2dec/strategy/grisu.rs:467
#1  core::ops::function::FnMut::call_mut () at library/core/src/ops/function.rs:150
#2  core::num::flt2dec::to_shortest_str () at library/core/src/num/flt2dec/mod.rs:397
#3  core::fmt::float::float_to_decimal_common_shortest () at library/core/src/fmt/float.rs:66
#4  0x000055555555a02c in core::fmt::write () at library/core/src/fmt/mod.rs:1149
#5  0x0000555555555784 in syscall_test::_start () at src/main.rs:31
(gdb) frame 0
#0  0x0000555555559d81 in core::num::flt2dec::strategy::grisu::format_shortest () at library/core/src/num/flt2dec/strategy/grisu.rs:467
467	in library/core/src/num/flt2dec/strategy/grisu.rs
(gdb) info locals
No locals.
```
It moved back to line 467, but still no variables in the debug symbols. Lets just clean and try with `debuginfo-level-std = 2` again. Well, seems to have gotten past the `Compiling object v0.26.2` line now... `./x.py build -i library/std`, [on the stages](https://rustc-dev-guide.rust-lang.org/building/bootstrapping.html#stages-of-bootstrapping).

Ok, we now have a new fresh stage1 compiler.
```
ivor@eagle:~/Documents/Code/rust/syscall_thing/syscall_test$ cargo +stage1 rustc -- --version
   Compiling syscall_test v0.0.0 (/home/ivor/Documents/Code/rust/syscall_thing/syscall_test)
rustc 1.60.0-dev
```
Issue moved back to 364;
```
Program received signal SIGSEGV, Segmentation fault.
0x0000555555559315 in core::num::flt2dec::to_shortest_str () at library/core/src/num/flt2dec/mod.rs:364
364	    let (negative, full_decoded) = decode(v);
(gdb) bt
#0  0x0000555555559315 in core::num::flt2dec::to_shortest_str () at library/core/src/num/flt2dec/mod.rs:364
#1  0x0000555555558fe2 in core::fmt::float::float_to_decimal_common_shortest () at library/core/src/fmt/float.rs:66
#2  0x000055555555705e in core::fmt::write () at library/core/src/fmt/mod.rs:1168
#3  0x0000555555556744 in syscall_test::_start () at src/main.rs:31
(gdb) info locals
No locals.
```

Maybe `debuginfo-level-core`, nope, that key results in a parse failure, much approve.

Ok, well gigabytes of debug symbols are better than no debug symbols, `debuginfo-level = 2` it is. Ok, well that definitely takes longer and makes my music glitch, good sign.

Well, 30 mins later, we have debug symbols (not in the first scope though, `v` is optimised out there)... rust directory now comes in at 44G. Maybe it actually worked before, but I only checked the first scope... since I expected `negative` to be a local scope thing there...
```
(gdb) up
#1  0x0000555555558382 in core::fmt::float::float_to_decimal_common_shortest (fmt=0x7fffffffdc90, num=<optimized out>, sign=core::num::flt2dec::Sign::Minus, precision=0) at library/core/src/fmt/float.rs:66
66	    let formatted = flt2dec::to_shortest_str(
(gdb) info locals
parts = [core::mem::maybe_uninit::MaybeUninit<core::num::fmt::Part> {uninit: (), value: core::mem::manually_drop::ManuallyDrop<core::num::fmt::Part> {value: core::num::fmt::Part}}, 
  core::mem::maybe_uninit::MaybeUninit<core::num::fmt::Part> {uninit: (), value: core::mem::manually_drop::ManuallyDrop<core::num::fmt::Part> {value: core::num::fmt::Part}}, 
  core::mem::maybe_uninit::MaybeUninit<core::num::fmt::Part> {uninit: (), value: core::mem::manually_drop::ManuallyDrop<core::num::fmt::Part> {value: core::num::fmt::Part}}, 
  core::mem::maybe_uninit::MaybeUninit<core::num::fmt::Part> {uninit: (), value: core::mem::manually_drop::ManuallyDrop<core::num::fmt::Part> {value: core::num::fmt::Part}}]
buf = [core::mem::maybe_uninit::MaybeUninit<u8> {uninit: (), value: core::mem::manually_drop::ManuallyDrop<u8> {value: 0}} <repeats 17 times>]
```

We finally have something, definitely an upgrade over no symbols. Segfault still happens, so that's good.

---

Trying to use the compiler builtins results in; 
```
ivor@eagle:~/Documents/Code/rust/syscall_thing/syscall_test$ cargo +stage1 r
   Compiling cc v1.0.72
   Compiling compiler_builtins v0.1.66 (https://github.com/rust-lang/compiler-builtins#ea0cb5b5)
error: failed to run custom build command for `compiler_builtins v0.1.66 (https://github.com/rust-lang/compiler-builtins#ea0cb5b5)`

Caused by:
  process didn't exit successfully: `/home/ivor/Documents/Code/rust/syscall_thing/syscall_test/target/debug/build/compiler_builtins-fbbb7e3cc9b25c66/build-script-build` (signal: 11, SIGSEGV: invalid memory reference)
```

to get the actual build thing; `cargo +stage1 b -Z unstable-options --build-plan` gives a json that's human readable!

Well, invoking that results in;
```
error: invalid `--cfg` argument: `feature=c` (expected `key` or `key="value"`)
```
Which... I couldn't get right without; https://github.com/rust-lang/rust/issues/66450#issue-523560575


Ok, I _think_ the problem is with this step;
```
rustc --crate-name build_script_build /home/ivor/.cargo/git/checkouts/compiler-builtins-79341f926ffc30b3/ea0cb5b/build.rs --error-format=json --json=diagnostic-rendered-ansi,future-incompat --crate-type bin --emit=dep-info,link -C embed-bitcode=no -C debuginfo=2 --cfg 'feature="c"' --cfg 'feature="cc"' --cfg 'feature="compiler-builtins"' --cfg 'feature="default"' --cfg 'feature="mem"' -C metadata=c77bccbbbf784625 -C extra-filename=-c77bccbbbf784625 --out-dir /home/ivor/Documents/Code/rust/syscall_thing/syscall_test/target/debug/build/compiler_builtins-c77bccbbbf784625 -L dependency=/home/ivor/Documents/Code/rust/syscall_thing/syscall_test/target/debug/deps --extern cc=/home/ivor/Documents/Code/rust/syscall_thing/syscall_test/target/debug/deps/libcc-c593bffe40f69992.rlib --cap-lints allow -C link-arg=-nostartfiles
```

There, we build the build script with `-nostartfiles`, but we need to be able to invoke the build script once build... with that `-C link-arg=-nostartfiles`, that won't work?

Manually invoking the various steps, where we build `compiler_builtins` without `-nostartfiles` seems to have built a working `-nostartfiles` version of main.rs, which doesn't complain about `memcpy` missing anymore...


Hmm, maybe we can use 

https://github.com/rust-lang/cargo/pull/9322/files
https://doc.rust-lang.org/cargo/reference/unstable.html#target-applies-to-host

to only apply the `-nostartfiles` to the actual final target? That is not working for `cargo build`, so we always need to specify the `--target x86_64-unknown-linux-gnu` flag.



---

Ok, but compiler-builtins doesn't compile with older rusts...

So we bring back our own memcpy, and start bisecting.

- +1.41.0-x86_64-unknown-linux-gnu -> works
- +1.45.0-x86_64-unknown-linux-gnu -> works
- +1.50.0-x86_64-unknown-linux-gnu -> works
- +1.55.0-x86_64-unknown-linux-gnu -> *fails*
- +1.52.0-x86_64-unknown-linux-gnu -> works
- +1.54.0-x86_64-unknown-linux-gnu -> *fails
- +1.53.0-x86_64-unknown-linux-gnu -> works

Works ends up spinning on the loop.

So failure is introduced somewhere between 1.53 and 1.54... Cool

1.53: https://github.com/rust-lang/rust/commit/53cb7b09b00cbea8754ffb78e7e3cb521cb8af4b

1.54: https://github.com/rust-lang/rust/commit/a178d0322ce20e33eac124758e837cbd80a6f633

Oh, those commits are outside the repo?

https://github.com/rust-lang/rust/compare/1.53.0...1.54.0

Well then... 5k+ commits.

https://github.com/rust-lang/cargo-bisect-rustc/blob/master/TUTORIAL.md

whoa, if that can actually grab the binaries from like 1.53... that all sounds amazing.

https://github.com/rust-lang/cargo-bisect-rustc/blob/master/TUTORIAL.md#testing-with-a-script

Let's make a script that returns `0` for success, and non zero for failure. So the segfault is 139, we just need a timeout on the loop.

This is an odd discovery; 

```
    fn write_str(&mut self, _s: &str) -> Result<(), Error> {
        panic!()
        // Ok(())
    }
```

Works with 1.53, but 
```
    fn write_str(&mut self, _s: &str) -> Result<(), Error> {
        // panic!()
        Ok(())
    }
```
still segfaults, even with 1.53... still in grisu. Maybe the issue moves around depending on how the code is written?

