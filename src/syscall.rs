// https://github.com/torvalds/linux/blob/v4.15/arch/x86/include/asm/syscall.h

// use core::arch;
use core::arch::asm; // in latest nightly.

// Oh, wow https://blog.rust-lang.org/inside-rust/2020/06/08/new-inline-asm.html
// that describes how to get the return value ven.

// Well, using alloc is a whole rabbit hole as we don't have malloc. No heap for now...
// #![feature(default_alloc_error_handler)]
// extern crate alloc;
// use alloc::string::String;

/// Exits the program with a return code.
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

        // Better to use the new unstable syntax
        // https://doc.rust-lang.org/beta/unstable-book/library-features/asm.html

        // So, set eax with the system call id
        // set the return code in ebx
        // trigger interrupt and hope for the best

        // With the legacy int 0x80 instruction;
        // let exit_syscall_id: u32 = 1;
        // asm!("int $$0x80", in("eax") exit_syscall_id, in("ebx") return_code);

        // Replaced with the syscall instruction
        const SYSCALL_ID: u32 = 60;
        asm!("syscall", in("rax") SYSCALL_ID, in("rdi") return_code,
            lateout("rcx") _,
            lateout("r11") _,
        );
    }
}

pub unsafe fn write(fd: u64, buffer: *const char, length: u64) {
    // where... is the syscall documentation?? >_<
    // Guess it's here; https://github.com/torvalds/linux/blob/v4.15/fs/read_write.c#L581-L596
    // So... some uint32 fd, const char* __user_string, size_t count.

    const SYSCALL_ID: u32 = 1; // write, in 64 bit syscall.

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
    asm!("syscall",
        in("rax") SYSCALL_ID,
        in("rdi") fd,
        in("rsi") buffer,
        in("rdx") length,
        lateout("rcx") _,
        lateout("r11") _,
    );
}
