#![no_std]
#![no_main]

// https://github.com/torvalds/linux/blob/v4.15/arch/x86/include/asm/syscall.h


// here we go :o
#![feature(asm)]
// use of unstable library feature 'asm': inline assembly is not stable enough for use and is subject to change
pub fn exit(return_code: i32) {
    // https://github.com/torvalds/linux/blob/v4.15/arch/x86/entry/syscalls/syscall_32.tbl
    let exit_syscall_id: u32 = 1;
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
        // trigger interrupt and hope for the best.
        asm!("int $$0x80", in("eax") exit_syscall_id, in("ebx") return_code);
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    exit(32);
    loop {}
}


use core::panic::PanicInfo;
#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    loop{}
}

