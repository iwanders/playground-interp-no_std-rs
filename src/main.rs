#![no_std]
// Even though we have main, we do all handling around main ourselves, so we don't want Rust to
// do anything around that.
#![no_main]
// here we go :o

extern crate syscall_test;

use syscall_test::io::*;
use syscall_test::{context, println, syscall::exit};

#[no_mangle]
pub fn main() -> ! {
    let context = context();
    context.dump();

    if context.is_interpreter() {
        println!("We are an interpreter, dispatching to main application entry.");
        context.entry();
    }

    // when this is called without checking is_interpreter, we endlessly ged the debug prints, so
    // that implies we're setting up rsp correctly again.
    // context.entry();
    syscall_test::test_all();
    // Lets exit gracefully.
    exit(0);
    unreachable!();
}

use core::panic::PanicInfo;
/// Handler for panic events, prints and exists using the syscall.
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    exit(99);
    unreachable!();
}
