
use crate::{fmt, fmt::Write, print_sstr, println, StackString};

pub extern crate core;

use core::mem::transmute;

use crate::support::strlen;

pub struct AbiContext
{
    pub rsp: *const u8,
}

impl AbiContext
{
    pub fn argc(&self) -> usize
    {
        // argc is at the location of rsp.
        unsafe {
            *transmute::<*const u8, *const usize>(self.rsp)
        }
    }

    pub fn argv(&self, index: usize) -> &[u8]
    {
        unsafe{
            let arg_ptr = transmute::<*const u8, *const u64>(self.rsp.offset((index + 1 ) as isize * 8));
            let arg = transmute::<u64, *const u8>(*arg_ptr);
            let len = strlen(arg, 1024);
            core::slice::from_raw_parts(arg, len)
        }
    }

    pub fn argv_str(&self, index: usize) -> Result<&str, core::str::Utf8Error>
    {
        core::str::from_utf8(self.argv(index))
    }

    pub fn dump(&self)
    {
        println!("rsp: {:?}", self.rsp);
        println!("argc: {:?}", self.argc());
        for i in 0..self.argc()
        {
            println!("arg{}: {:?}", i, self.argv(i));
            println!("arg_str{}: {:?}", i, self.argv_str(i));
        }

    }
}

static mut original_rsp: u64 = 0;


pub fn context() -> AbiContext
{
    return AbiContext{rsp: unsafe {transmute::<u64, *const u8>(crate::abi::original_rsp)}};
}

// state that there will be a main.
extern "C" {
    pub fn main();
}

#[no_mangle]
pub unsafe extern "C" fn _start_stage2()
{
    // Rdi was stored from rsp in _start, so here we can really read it, and store it for 
    // posterity.
    core::arch::asm!("nop", out("rdi") original_rsp);

    // Then, we can go into main itself.
    main();
}

#[no_mangle]
#[naked] // disable prologue; https://github.com/nox/rust-rfcs/blob/master/text/1201-naked-fns.md
pub unsafe extern "C" fn _start(){
    core::arch::asm!("mov rdi, rsp
    // invoke main.
    call _start_stage2",
            options(noreturn));  // can't read rsp into original_rsp here, as it is zero, or just not used?
}

