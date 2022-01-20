#![no_std]
#![feature(naked_functions)]
// here we go :o

mod abi;
pub mod io;
mod support;
pub mod syscall;
mod util;

pub use abi::context;
pub use abi::AbiContext;
pub mod fs;

pub mod mem;

pub fn test_all() {
    crate::fs::test::test_all();
    crate::mem::test::test_all();
    crate::syscall::test::test_all();
}
