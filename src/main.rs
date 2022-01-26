#![no_std]
// Even though we have main, we do all handling around main ourselves, so we don't want Rust to
// do anything around that.
#![no_main]
// here we go :o

pub fn _start()
{
    loop{}
}

use core::panic::PanicInfo;
/// Handler for panic events, prints and exists using the syscall.
#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
