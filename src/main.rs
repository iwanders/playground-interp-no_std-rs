#![no_std]
#![no_main]

// here we go :o
// #![feature(asm)]

// use syscall_test;
extern crate syscall_test;
use ::syscall_test::{exit, print};

use ::syscall_test::{println, fmt, StackString, print_sstr, fmt::Write};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // write();
    // print("hello");
    // printauxv();
    // printb("b");
    // print("hello");
    // println!("{} haha", 1);

    for i in 0..10
    {
        // println!("Lorem {} ipsum {:?} dolor {} ", 5, Some(i), "foo");
        println!("Lorem {} ipsum {:?} dolor {}sdifjdsifjdslkfjlksdjflksdjflkdsjlkf ", 5, Some(i), "foo");
        // println!("Lorem {} ipsum {:?} dolor {}sdifjdsifjdslkfjlksdjflksdjflkdsjlkf ", 5, Some(3.3), "foo");
    }

    // stackallocate();

    exit(33);

    for _i in 0..100000
    {
        print(".");
    }
    // lets do some stack exhaustion and see where it fails...
    // recurser(0);
    // x0000555555554505 in test::recurser (z=262007) at src/main.rs:218
    // 1048028 bytes... that's not too bad... sounds like my stack string should also work??

    // Unless that... grows from the other side or something?

    print("z");
    print("x");

    exit(33);
    loop {}
}

use core::panic::PanicInfo;
#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    print("\nPanic!");
    exit(99);
    loop{};
}
