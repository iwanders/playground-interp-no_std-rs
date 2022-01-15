use crate::io::*;
use crate::println;

pub extern crate core;

use core::mem::transmute;

use crate::support::strlen;

pub struct AbiContext {
    pub rsp: *const u8,
}

impl AbiContext {
    pub fn new(rsp: *const u8) -> Self {
        AbiContext { rsp }
    }
    pub fn argc(&self) -> usize {
        // argc is at the location of rsp.
        unsafe { *transmute::<*const u8, *const usize>(self.rsp) }
    }

    pub fn argv_bytes(&self, index: usize) -> &[u8] {
        if index >= self.argc() {
            panic!("Requested argv beyond argc.");
        }
        unsafe {
            let arg_ptr =
                transmute::<*const u8, *const u64>(self.rsp.offset((index + 1) as isize * 8));
            let arg = transmute::<u64, *const u8>(*arg_ptr);
            let len = strlen(arg, 1024);
            core::slice::from_raw_parts(arg, len)
        }
    }

    pub fn argv(&self, index: usize) -> Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(self.argv_bytes(index))
    }

    pub fn dump(&self) {
        println!("rsp: {:?}", self.rsp);
        println!("argc: {:?}", self.argc());
        for i in 0..self.argc() {
            // println!("argv_bytes{}: {:?}", i, self.argv_bytes(i));
            println!("argv{}: {:?}", i, self.argv(i));
        }
    }
}

static mut ORIGINAL_RSP: u64 = 0;

pub fn context() -> AbiContext {
    return AbiContext::new(unsafe { transmute::<u64, *const u8>(crate::abi::ORIGINAL_RSP) });
}

// state that there will be a main.
extern "C" {
    pub fn main();
}

#[no_mangle]
pub unsafe extern "C" fn _start_stage2() {
    // Rdi was stored from rsp in _start, so here we can really read it, and store it for
    // posterity.
    core::arch::asm!("nop", out("rdi") ORIGINAL_RSP);

    // Then, we can go into main itself.
    main();
}

#[no_mangle]
#[naked] // disable prologue; https://github.com/nox/rust-rfcs/blob/master/text/1201-naked-fns.md
pub unsafe extern "C" fn _start() {
    core::arch::asm!(
        "mov rdi, rsp
    // invoke main.
    call _start_stage2",
        options(noreturn)
    ); // can't read rsp into original_rsp here, as it is zero, or just not used?
}
