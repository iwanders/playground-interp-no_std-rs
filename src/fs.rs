use crate::syscall::{open, write, read};

pub fn read(path: &str) -> Result<Vec<u8>>
    let binary_blob = fs::read("test/test").expect("Can't read binary");
