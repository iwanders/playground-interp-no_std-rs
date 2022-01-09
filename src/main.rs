#![no_std]
#![no_main]

// Need to provide memcmp, memcpy and memset.
// Setting breakpoints in these functions, or changing them into loop{}
// shows they are not used by the code below, they're merely here to satisfy the linker.
mod support;


// We need something that implements Write, segfault happens before this, so we don't need to
// actually implement anything properly.
struct WritableThing{}
use core::fmt::Error;
impl fmt::Write for WritableThing {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        panic!()
    }
}

// Import fmt from the core crate.
use core::fmt;


// The entry point to our binary.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Create something writeable.
    let mut v: WritableThing = WritableThing{};

    // Try to format a float.
    fmt::write(&mut v, format_args!("{}", 3.3)).expect("Error occurred while trying to write in String");

    // We'll never get here, but we go into this loop to check whether we got past the format with
    // gdb.
    loop {}
}


// Because we have no std we need to provide a panic handler.
use core::panic::PanicInfo;
#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
