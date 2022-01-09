

```
(gdb) info proc mappings
process 22376
Mapped address spaces:

          Start Addr           End Addr       Size     Offset objfile
      0x555555554000     0x555555558000     0x4000        0x0 /home/ivor/Documents/Code/rust/syscall_thing/syscall_test/target/debug/test
      0x555555757000     0x555555758000     0x1000     0x3000 /home/ivor/Documents/Code/rust/syscall_thing/syscall_test/target/debug/test
      0x7ffff7dd3000     0x7ffff7dfc000    0x29000        0x0 /lib/x86_64-linux-gnu/ld-2.27.so
      0x7ffff7ff5000     0x7ffff7ff7000     0x2000        0x0 
      0x7ffff7ff7000     0x7ffff7ffa000     0x3000        0x0 [vvar]
      0x7ffff7ffa000     0x7ffff7ffc000     0x2000        0x0 [vdso]
      0x7ffff7ffc000     0x7ffff7ffe000     0x2000    0x29000 /lib/x86_64-linux-gnu/ld-2.27.so
      0x7ffff7ffe000     0x7ffff7fff000     0x1000        0x0 
      0x7ffffffde000     0x7ffffffff000    0x21000        0x0 [stack]
  0xffffffffff600000 0xffffffffff601000     0x1000        0x0 [vsyscall]
```

So, from the looks of it... our stack is 0x21000, which is 135168 bytes
```
(gdb) bt
#0  test::memset (ptr=0x7fffffffddce "\000\000\070\xffffffde\177\000\000\000\020", fill=0 '\0', size=50) at src/main.rs:191
#1  0x00005555555558fe in <test::StackString as core::default::Default>::default () at src/main.rs:149
#2  0x0000555555555e6b in test::println (input=...) at src/main.rs:205
#3  0x0000555555555fa6 in test::_start () at src/main.rs:226
```

So, from the looks of it... our stack is 0x21000, which is 135168 bytes
// x0000555555554505 in test::recurser (z=262007) at src/main.rs:218
// 1048028 bytes... that's not too bad... sounds like my stack string should also work??

// The first regions of 0x4000 and 0x1000 are probably static data such as my static strings?

// Ok, the stack automatically grows, with the recurser function it is at 
```
      0x7fffffb6a000     0x7ffffffff000   0x495000        0x0 [stack]
```

Then, the question remains why does my string allocation / memset segfault?

// Oh, maybe my syscall eliminates the stack return address for when we jump out of println?

Marking everything as clobbered doesn't seem to do anything, neither does saving / restoring rsp.


Ok... disabled memcpy and memset for now... doing a float formatting segfaults, small strings seem to work?


--
Aha, valgrind says we do jump to address 0, and then segfault.

Ah, memset takes fill as i32, not u8... that was that difference in signature caused the jump to 0x00.

--
Formatting floats still bails, but that's hardly a problem for now:

        println!("Lorem {} ipsum {:?} dolor {}sdifjdsifjdslkfjlksdjflksdjflkdsjlkf ", 5, Some(3.3), "foo");

```
=5142== Process terminating with default action of signal 11 (SIGSEGV)
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

Crash in grisu disappears when we do;
```
cargo b -Z build-std=core --target x86_64-unknown-linux-gnu
```

We do have to provide memcmp in latest nightly.
