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

        // Make a variable we'll point at the start of the elf file.
        let mut elf_file_start: Option<*const u8> = None;

        for i in 0u64..phnum {
            let sl;
            let part_start = unsafe { phdr.offset((phent * i) as isize) };
            unsafe {
                sl = core::slice::from_raw_parts(part_start, (phent * 1) as usize);
            }
            let phdr_parsed = xmas_elf::program::ProgramHeader::Ph64(read(sl));
            println!("{:?}", phdr_parsed);

            // The dynamic data is the interesting part.
            if let Ok(v) = phdr_parsed.get_type() {
                // Program header itself contains the offset in the elf file, so we can
                // subtract the offset from the current address to find the start of the
                // ELF file itself.
                if v == xmas_elf::program::Type::Phdr {
                    elf_file_start = Some(unsafe{part_start.offset(-(phdr_parsed.offset() as isize))});
                    println!("elf_file_start: {:?}", elf_file_start);
                }
                // Phdr is the first entry, so we can just use elf_file_start from here on.
                if v == xmas_elf::program::Type::Dynamic {

                    let get_chunk = |pos: u64, length: u64| -> &[u8] {
                        unsafe {
                            core::slice::from_raw_parts(
                                (elf_file_start.as_ref().expect("should have elf_file start").offset(pos as isize)) as *const u8,
                                length as usize,
                            )
                        }
                    };
                    let elf_file_start = elf_file_start.as_ref().expect("Should have elf file start.");
                    // Then, parse the segment data.
                    let segment_data_parsed = xmas_elf::program::SegmentData::Dynamic64(
                        read_array(get_chunk(phdr_parsed.offset(), phdr_parsed.mem_size())),
                    );
                    // And do something if the segment data is dynamic64.
                    if let xmas_elf::program::SegmentData::Dynamic64(segments) = segment_data_parsed
                    {
                        println!("segment: {:?}", segments);
                        for segment in segments {
                            println!("Entry : {:?}", segment);
                            match segment.get_tag().expect("Will succeed") {
                                xmas_elf::dynamic::Tag::Pltgot => {
                                    println!("Pltgot ptr : {:?}", segment.get_ptr());
                                },
                                xmas_elf::dynamic::Tag::PltRelSize => {
                                    println!("PltRelSize : {:?}", segment.get_val());
                                },
                                _ => {},
                            }
                        }
                    }
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
