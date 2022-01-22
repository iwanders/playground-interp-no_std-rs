#![no_std]
#![no_main]
#![feature(naked_functions)]

// Need to provide memcmp, memcpy and memset.
// Setting breakpoints in these functions, or changing them into loop{}
// shows they are not used by the code below, they're merely here to satisfy the linker.
mod support;
// extern crate compiler_builtins;

// We need something that implements Write, segfault happens before this, so we don't need to
// actually implement anything properly.
struct WritableThing{}
use core::fmt::Error;
impl fmt::Write for WritableThing {
    fn write_str(&mut self, _s: &str) -> Result<(), Error> {
        // panic!()
        Ok(())
    }
}

// Import fmt from the core crate.
use core::fmt;


// The entry point to our binary.
fn main() -> ! {
    // Create something writeable.
    let mut v: WritableThing = WritableThing{};

    // Try to format a float.
    fmt::write(&mut v, format_args!("{}", 3.3)).expect("Error occurred while trying to write in String");

    // We'll never get here, but we go into this loop to check whether we got past the format with
    // gdb.
    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn _start_stage2() {
    // Rdi was stored from rsp in _start, so here we can really read it, and store it for
    // posterity.

    // Then, we can go into main itself.
    main();
}

/// The entry point of our program, naked function to prevent the prologue, this function copies
/// $rsp into rdx and then calls the stage 2 start function.
#[no_mangle]
#[naked] // disable prologue; https://github.com/nox/rust-rfcs/blob/master/text/1201-naked-fns.md
pub unsafe extern "C" fn _start() {
    core::arch::asm!(
        // "mov rdi, rsp
    "
    // invoke main.
    call _start_stage2",
        options(noreturn)
    ); // can't read rsp into original_rsp here, as it is zero, or just not used?
}

// Because we have no std we need to provide a panic handler.
use core::panic::PanicInfo;
#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
