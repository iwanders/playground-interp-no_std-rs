#![no_std]
// Even though we have main, we do all handling around main ourselves, so we don't want Rust to
// do anything around that.
#![no_main]
// here we go :o

extern crate syscall_test;
extern crate xmas_elf;
extern crate zero;

// Import our implementations to exit, print and obtain the abi context.
use syscall_test::io::*;
use syscall_test::syscall;
use syscall_test::{context, println};

mod dynamic_linker {
    use super::*;
    use syscall_test::fs;
    use syscall_test::syscall;
    use xmas_elf;
    use zero::{read, read_array, Pod};

    pub fn link(context: &syscall_test::AbiContext) -> Result<(), &'static str> {
        // AT_EXECFD holds the file descriptor to the main program memory OR
        // AT_PHDR, with AT_PHENT and AT_PHNUM may be set to specify the program headers.
        let (phdr, phent, phnum) = context.get_phdr().ok_or("No phdr?")?;

        // Before the program header is the first header part...
        let file;
        unsafe {
            let z = phdr.offset(-(core::mem::size_of::<xmas_elf::header::HeaderPt1>() as isize) - 48);
            let full_file = core::slice::from_raw_parts(
                z,
                core::mem::size_of::<xmas_elf::header::HeaderPt1>() + (phent * phnum) as usize + 10000,
            );
            // println!("{:x?}", full_file);
            file = xmas_elf::ElfFile::new(full_file).expect("SHould pass");
        }

        for i in 0u64..phnum {
            let sl;
            let part_start = unsafe{phdr.offset((phent * i) as isize)};
            unsafe {
                sl = core::slice::from_raw_parts(
                    part_start,
                    (phent * 1) as usize,
                );
            }
            let phdr_parsed = xmas_elf::program::ProgramHeader::Ph64(read(sl));
            println!("{:?}", phdr_parsed);

            // The dynamic data is the interesting part.
            if let Ok(v) = phdr_parsed.get_type()
            {
                if v == xmas_elf::program::Type::Dynamic
                {
                    let zzzz = phdr_parsed.get_data(&file);
                    
                    let data_sl;
                    unsafe {
                        data_sl = core::slice::from_raw_parts(
                            phdr.offset(phdr_parsed.offset() as isize),
                            (phdr_parsed.mem_size()) as usize,
                        );
                    }
                    // let data = xmas_elf::sections::SectionData::Dynamic64(read_array(data_sl));
                    println!("{:?}", zzzz);
                }
            }
        }
        Ok(())
    }
}

#[no_mangle]
pub fn main() -> ! {
    let context = context();
    context.dump();

    // Here, something with dynamic linking...
    if context.is_interpreter() {
        println!("We are an interpreter, kernel has loaded the binary, do the dynamic linking.");
        let res = dynamic_linker::link(&context);
        println!("{:?}", res);
        // context.entry();
    } else {
        println!("We are not an interpreter.");
        if context.argc() < 2 {
            println!("No file to load specified, bailing out.");
            syscall::exit(1);
        }
        println!("Got passed a file, but we don't know how to handle that.");
        syscall::exit(1);
    }

    // Lets exit gracefully if we ever get here.
    syscall::exit(0);
    unreachable!();
}

use core::panic::PanicInfo;
/// Handler for panic events, prints and exists using the syscall.
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    syscall::exit(99);
    unreachable!();
}
