#![no_std]
#![feature(naked_functions)]
// here we go :o

mod abi;
pub mod io;
mod support;
mod syscall;

pub use abi::context;

pub use syscall::{exit, write};
