#![no_std]
// here we go :o
#![feature(asm)]
mod syscall;
pub use crate::syscall::{exit, write};

pub fn print(input: &str) {
    unsafe {
        let f = input.as_ptr() as *const char;
        write(1, f, input.len() as u64);
    }
}

// use core::fmt;
pub struct StackString {
    pub buffer: [u8; 5000],
    pub size: usize,
}
impl StackString {
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
            buffer: [0; 5000],
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
// Well, format just assumes that memset and memcpy exist.
// We can definitely provide those to make the linker happy, they don't require allocations.
#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, size: usize) -> *mut u8 {
    let mut i = 0;
    // for i in 0..size
    while i < size {
        *dest.offset(i as isize) = *src.offset(i as isize);
        i += 1;
    }
    dest
}
#[no_mangle]
pub unsafe extern "C" fn memset(ptr: *mut u8, fill: i32, size: usize) -> *mut u8 {
    let mut i = 0;
    while i < size {
        (*ptr.offset(i as isize)) = fill as u8;
        i += 1;
    }
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, size: usize) -> i32 {
    let mut i = 0;
    while i < size {

        if *s1.offset(i as isize) != *s2.offset(i as isize)
        {
            return 1; // something about the sign...
        }
        i += 1;
    }
    0
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
        let mut v: StackString = StackString{buffer: [0; 5000], size: 0};
        // let mut v: StackString = Default::default();
        fmt::write(&mut v, format_args!($($arg)*)).expect("Error occurred while trying to write in String");
        v.write_str("\n").expect("Shouldn't fail");
        print_sstr(&v);
    })
}

pub fn stackallocate() {
    let mut _v: StackString = StackString {
        buffer: [0; 5000],
        size: 0,
    };
}
