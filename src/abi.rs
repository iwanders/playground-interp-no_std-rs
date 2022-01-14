
use crate::{fmt, fmt::Write, print_sstr, println, StackString};

pub extern crate core;

use core::mem::transmute;

pub struct AbiContext
{
    pub rsp: *const u8,
}

impl AbiContext
{
    pub fn argc(&self) -> u64
    {
        // argc is at the location of rsp.
        unsafe {
            *transmute::<*const u8, *const u64>(self.rsp)
        }
    }
    pub fn dump(&self)
    {
        println!("rsp: {:?}", self.rsp);
        println!("argc: {:?}", self.argc());
    }
}


pub fn context() -> AbiContext
{
    let mut rsp: u64 = 0;
    unsafe
    {
        core::arch::asm!("mov rdi, rsp", out("rdi") rsp);
    }
    println!("0x{:x}", rsp);
    // context();
    return AbiContext{rsp: unsafe {transmute::<u64, *const u8>(rsp)}};
}
