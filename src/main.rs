#![no_std]
#![no_main]

// here we go :o
// #![feature(asm)]

// use syscall_test;
extern crate syscall_test;
use syscall_test::{exit, print};

use syscall_test::{fmt, fmt::Write, print_sstr, println, StackString};

use syscall_test::abi::{context};


static mut rsp: u64 = 0;

#[no_mangle]
pub extern "C" fn _start() -> ! {

    // let mut rsp: u64 = 0;
    unsafe
    {
        core::arch::asm!("mov rdi, rsp", out("rdi") rsp);
        println!("0x{:x}", rsp);
    }
/*
0000000000400180 <_start>:
  400180:	b8 a8 21 00 00       	mov    $0x21a8,%eax
  400185:	e8 2d 73 02 00       	callq  4274b7 <__rust_probestack>
  40018a:	48 29 c4             	sub    %rax,%rsp
  40018d:	48 89 e7             	mov    %rsp,%rdi
  400190:	48 89 3d 69 ee 22 00 	mov    %rdi,0x22ee69(%rip)        # 62f000 <__bss_start>

rsp really is: 0x7fffffffdcd0
then jump over callq
sub
rsp now is:    0x7fffffffbb28
Then we copy rsp to rdi, at which it becomes 0x7fffffffbb28

and that gets stored into variable rsp. So basically, bummer, this rust probestack pretty much
ruins our day here... guess _start is usually written in assembly to avoid such issues?
*/

    let context = context();
    context.dump();

    // write();
    // print("hello");
    // printauxv();
    // printb("b");
    // print("hello");
    // println!("{} haha", 1);

    for i in 0..10 {
        // println!("Lorem {} ipsum {:?} dolor {} ", 5, Some(i), "foo");
        println!("Lorem {} ipsum {:?} dolor {:?} ", 5, Some(i), Some(3.3));
        // println!("Lorem {} ipsum {:?} dolor {} ", 5, Some(3.3), "foo");
    }

    // stackallocate();

    // exit(0);

    for _i in 0..1 {
        print(".");
    }
    // lets do some stack exhaustion and see where it fails...
    // recurser(0);
    // x0000555555554505 in test::recurser (z=262007) at src/main.rs:218
    // 1048028 bytes... that's not too bad... sounds like my stack string should also work??

    // Unless that... grows from the other side or something?

    print("z");
    print("x\n");

    exit(0);
    loop {}
}

use core::panic::PanicInfo;
#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    print("\nPanic!");
    exit(99);
    loop {}
}
