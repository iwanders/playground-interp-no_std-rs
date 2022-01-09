#![no_std]
// here we go :o
#![feature(asm)]
mod syscall;
mod support;
pub use crate::syscall::{exit, write};

pub fn print(input: &str) {
    unsafe {
        let f = input.as_ptr() as *const char;
        write(1, f, input.len() as u64);
    }
}

const StackStringSize: usize = 4096;
pub struct StackString {
    pub buffer: [u8; StackStringSize],
    pub size: usize,
}
impl StackString {
    const StackStringSize: usize = StackStringSize;
    fn as_ptr(&self) -> *const char {
        self.buffer.as_ptr() as *const char
    }
    fn len(&self) -> usize {
        self.size
    }
}

impl Default for StackString {
    fn default() -> Self {
        StackString {
            buffer: [0; StackStringSize],
            size: 0,
        }
    }
}

use core::cmp::min;
use core::fmt::Error;
impl fmt::Write for StackString {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for i in 0..min(s.len(), self.buffer.len() - self.size) {
            self.buffer[self.size] = s.as_bytes()[i];
            self.size += 1;
        }
        if self.size == self.buffer.len() {
            return Err(Error {});
        }
        Ok(())
    }
    // fn write_char(&mut self, c: char) -> Result { ... }
    // fn write_fmt(&mut self, args: Arguments<'_>) -> Result { ... }
}

// use core::fmt;
pub fn print_sstr(input: &StackString) {
    unsafe {
        let f = input.as_ptr() as *const char;
        let l = input.len() as u64;
        write(1, f, l);
    }
}

pub use core::fmt;
// Adopted from https://doc.rust-lang.org/src/std/macros.rs.html#94-99
#[macro_export]
macro_rules! println {
    () => (print("\n"));
    ($($arg:tt)*) => ({
        let mut v: StackString = Default::default();
        // let mut v: StackString = Default::default();
        fmt::write(&mut v, format_args!($($arg)*)).expect("Error occurred while trying to write in String");
        v.write_str("\n").expect("Shouldn't fail");
        print_sstr(&v);
    })
}
