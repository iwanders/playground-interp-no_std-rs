#![no_std]
#![no_main]
// https://github.com/torvalds/linux/blob/v4.15/arch/x86/include/asm/syscall.h

// here we go :o
#![feature(asm)]


// Well, using alloc is a whole rabbit hole as we don't have malloc. No heap for now...
// #![feature(default_alloc_error_handler)]
// extern crate alloc;
// use alloc::string::String;

// Exits the problam with a return code.
pub fn exit(return_code: i32) {
    // https://github.com/torvalds/linux/blob/v4.15/arch/x86/entry/syscalls/syscall_32.tbl
    unsafe {
        /*
            https://github.com/torvalds/linux/blob/v4.15/arch/x86/entry/entry_64_compat.S#L289
            32-bit legacy system call entry:
            Arguments:
            eax  system call number
            ebx  arg1... and a few more args in various registers.
        */

        // The following doc seems to be outdated?
        // https://doc.rust-lang.org/1.8.0/book/inline-assembly.html
        // Direct binding to https://llvm.org/docs/LangRef.html#inline-assembler-expressions
        // Claim's it is;
        /*
            asm!(assembly template
            : output operands
            : input operands
            : clobbers
            : options
            );

            asm!("add $2, $0"
                 : "=r"(c)
                 : "0"(a), "r"(b)
                 );
        */

        // Better to use
        // https://doc.rust-lang.org/beta/unstable-book/library-features/asm.html

        // So, set eax with the system call id
        // set the return code in ebx
        // trigger interrupt and hope for the best

        // let exit_syscall_id: u32 = 1;
        // asm!("int $$0x80", in("eax") exit_syscall_id, in("ebx") return_code);

        // Replaced with the syscall instruction
        const SYSCALL_ID: u32 = 60;
        asm!("syscall", in("rax") SYSCALL_ID, in("rdi") return_code,
            lateout("rax") _,
            lateout("rdi") _,
        );
    }
}

unsafe fn write(fd: u64, buffer: *const char, length: u64) {
    // where... is the syscall documentation?? >_<
    // Guess it's here; https://github.com/torvalds/linux/blob/v4.15/fs/read_write.c#L581-L596
    // So... some uint32 fd, const char* __user_string, size_t count.

    // Probably don't need the string terminator... but probably good practice?
    // let z = "booo\n";
    // let size: u64 = z.len() as u64;
    // const FD: u64 = 1;

    // let z = format!("{}", 3);
    const SYSCALL_ID: u32 = 1; // write, in 64 bit syscall.

    // So, presumably, this string is passed in as a pointer, that's my hunch at least.
    // stdout is 1.

    // unsafe {
    // let f = &Z as *const char;
    // asm!("int $$0x80", in("eax") SYSCALL_ID, in("ebx") FD, in("ecx") f, in("edx") SIZE);
    // }
    // Well, that doesn't work, it also doesn't crash.
    // Oh, since size is 64 bits... we probably can't use the interrupt thing anymore as the edx
    // register is 32 bits or something?
    // https://github.com/torvalds/linux/blob/v3.13/arch/x86/syscalls/syscall_64.tbl

    // Page 670 of https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-2b-manual.pdf
    // describes the syscall assembly instruction :o

    // https://github.com/torvalds/linux/blob/v4.15/arch/x86/entry/entry_64.S#L107-L143
    /*
     * rax  system call number
     * rcx  return address
     * r11  saved rflags (note: r11 is callee-clobbered register in C ABI)
     * rdi  arg0
     * rsi  arg1
     * rdx  arg2
     * r10  arg3 (needs to be moved to rcx to conform to C ABI)
     * r8   arg4
     * r9   arg5
     */
    unsafe {
        // let f = z.as_ptr() as *const char;
        asm!("syscall",
            in("rax") SYSCALL_ID,
            in("rdi") fd,
            in("rsi") buffer,
            in("rdx") length,
            lateout("rcx") _,
            lateout("r11") _,
            lateout("rax") _,
            lateout("rdi") _,
        );
    }
}

fn print(input: &str) {
    unsafe {
        let f = input.as_ptr() as *const char;
        write(1, f, input.len() as u64);
    }
}



// use core::fmt;
struct StackString
{
    buffer: [u8; 5],
    size: usize,
}
impl StackString
{
    fn as_ptr(&self) -> *const char
    {
        self.buffer.as_ptr() as *const char
    }
    fn len(&self) -> usize
    {
        self.size
    }
}

impl Default for StackString
{
    fn default() -> Self
    {
        StackString{buffer: [0; 5], size: 0}
    }

}


use core::fmt::Error;
use core::cmp::min;
impl fmt::Write for StackString
{
    fn write_str(&mut self, s: &str) -> Result<(), Error>
    {
        for i in 0..min(s.len(), self.buffer.len() - self.size)
        {
            self.buffer[self.size] = s.as_bytes()[i];
            self.size += 1;
        }
        if (self.size == self.buffer.len())
        {
            return Err(Error{});
        }
        Ok(())
    }
}
    // fn write_char(&mut self, c: char) -> Result { ... }
    // fn write_fmt(&mut self, args: Arguments<'_>) -> Result { ... }
// }
// Well, format just assumes that memset and memcpy exist.
// We can definitely provide those to make the linker happy, they don't require allocations.

#[no_mangle]
pub unsafe fn memcpy(dest: *mut char, src: *const char, size: usize) {
    for i in 0..size
    {
        (*dest.offset(i as isize)) = *src.offset(i as isize);
    }
}
#[no_mangle]
pub unsafe fn memset(ptr: *mut char, fill: char, size: usize) {
    for i in 0..size
    {
        print(".");
        (*ptr.offset(i as isize)) = fill;
    }
}
// for some reason... some 

// #[no_mangle]
// pub unsafe fn malloc(size: usize) {
    // print("Malloc");
// }

use core::fmt;
fn println(input: &str) {
    // let with_newline = input;
    // let mut v: StackString = StackString{buffer: ['\x00'; 50], size: 0};
    let mut v: StackString = Default::default();
    fmt::write(&mut v, format_args!("{}", input))
        .expect("Error occurred while trying to write in String");
    unsafe {
        let f = v.as_ptr() as *const char;
        let l = v.len() as u64;
        write(1, f, l);
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // write();
    print("hello");
    // print("hello");
    println("ff");
    exit(33);
    loop {}
}

use core::panic::PanicInfo;
#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    print("\nPanic!");
    exit(99);
    loop{};
}
