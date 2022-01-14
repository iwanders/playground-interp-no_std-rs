#![no_std]
#![no_main]
#![feature(naked_functions)]
// here we go :o
// #![feature(asm)]

// use syscall_test;
extern crate syscall_test;
use syscall_test::{exit, print};

use syscall_test::{fmt, fmt::Write, print_sstr, println, StackString};

use syscall_test::{abi::context};



#[no_mangle]
pub fn main() -> ! {

    let context = context();
    context.dump();
    /*

*/
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
