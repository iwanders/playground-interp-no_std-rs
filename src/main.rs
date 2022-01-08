#![no_std]
#![no_main]

#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}


use core::panic::PanicInfo;
#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    loop{}
}

