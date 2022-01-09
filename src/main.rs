#![no_std]
#![no_main]

mod support;

struct WritableThing{}
use core::fmt::Error;
impl fmt::Write for WritableThing {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        Ok(())
    }
}

use core::fmt;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut v: WritableThing = WritableThing{};
    fmt::write(&mut v, format_args!("{}", 3.3)).expect("Error occurred while trying to write in String");
    loop {}
}

use core::panic::PanicInfo;
#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
