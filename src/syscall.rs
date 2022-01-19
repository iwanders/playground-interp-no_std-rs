// https://github.com/torvalds/linux/blob/v4.15/arch/x86/include/asm/syscall.h

// use core::arch;
use core::arch::asm; // in latest nightly.

// Oh, wow https://blog.rust-lang.org/inside-rust/2020/06/08/new-inline-asm.html
// that describes how to get the return value even.

// Well, using alloc is a whole rabbit hole as we don't have malloc. No heap for now...
// #![feature(default_alloc_error_handler)]
// extern crate alloc;
// use alloc::string::String;

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
// The following doc seems to be outdated?
// https://doc.rust-lang.org/1.8.0/book/inline-assembly.html

// Better to use the new unstable syntax
// https://doc.rust-lang.org/beta/unstable-book/library-features/asm.html

/// Exits the program with a return code.
pub fn exit(return_code: i32) -> ! {
    unsafe {
        const SYSCALL_ID: u32 = 60;
        asm!("syscall", in("rax") SYSCALL_ID, in("rdi") return_code,
            lateout("rcx") _,
            lateout("r11") _,
        );
    }
    unreachable!();
}

unsafe fn syscall_1_arg(syscall_id: u32, arg0: u64) -> u64 {
    let ret: u64;
    asm!("syscall",
        in("rax") syscall_id,
        in("rdi") arg0,
        lateout("rsi") _,
        lateout("rdx") _,
        lateout("rcx") _,
        lateout("r11") _,
        lateout("rax") ret,
    );
    ret
}
unsafe fn syscall_3_arg(syscall_id: u32, arg0: u64, arg1: u64, arg2: u64) -> u64 {
    let ret: u64;
    asm!("syscall",
        in("rax") syscall_id,
        in("rdi") arg0,
        in("rsi") arg1,
        in("rdx") arg2,
        lateout("rcx") _,
        lateout("r11") _,
        lateout("rax") ret,
    );
    ret
}

pub unsafe fn write(fd: u64, buffer: *const u8, length: u64) -> u64 {
    // where... is the syscall documentation?? >_<
    // Guess it's here; https://github.com/torvalds/linux/blob/v4.15/fs/read_write.c#L581-L596
    // So... some uint32 fd, const char* __user_string, size_t count.
    const SYSCALL_ID: u32 = 1; // write, in 64 bit syscall.
    syscall_3_arg(SYSCALL_ID, fd as u64, buffer as u64, length as u64)
}

pub unsafe fn read(fd: u64, buffer: *mut u8, length: u64) -> u64 {
    // https://github.com/torvalds/linux/blob/v4.15/fs/read_write.c#L566-L579
    // read, unsigned int, fd, char __user *, buf, size_t, count)
    const SYSCALL_ID: u32 = 0;
    syscall_3_arg(SYSCALL_ID, fd as u64, buffer as u64, length as u64)
}

/// To create a null terminated string, in a clunky way.
pub fn str_to_zero_padded(path: &str) -> [u8; 1024] {
    let mut path_buffer: [u8; 1024] = [0; 1024];
    for i in 0..path.len() {
        path_buffer[i] = path.as_bytes()[i];
    }
    path_buffer
}

// Modes
pub const O_RDONLY: i32 = 0o00000000;
pub const O_WRONLY: i32 = 0o00000001;
pub const O_RDWR: i32 = 0o00000002;

// Flags
pub const O_CREAT: i32 = 0o00000100;
pub const O_TRUNC: i32 = 0o00001000;
pub const O_APPEND: i32 = 0o00002000;

pub unsafe fn open(path: &str, flags: i32, umode: u16) -> u64 {
    // https://github.com/torvalds/linux/blob/v4.15/fs/open.c#L1072-L1078
    // const char __user *, filename, int, flags, umode_t, mode
    const SYSCALL_ID: u32 = 2;
    // syscall_3_arg(SYSCALL_ID, fd as u64, buffer as u64, length as u64)
    // Need a zero padded string.
    let path_buffer = str_to_zero_padded(&path);
    let path_p: *const u8 = &(path_buffer[0]) as *const u8;
    syscall_3_arg(SYSCALL_ID, path_p as u64, flags as u64, umode as u64)
}

pub unsafe fn close(fd: u64) -> u64 {
    const SYSCALL_ID: u32 = 3;
    syscall_1_arg(SYSCALL_ID, fd as u64)
}

type Stat = [u8; 144];
fn get_filesize(stat: &Stat) -> i64 {
    // let mut stat_struct: Stat = [0; 144];
    // filesize is at 48 bytes in.
    // filesize is 8 bytes itself. And it is signed.
    unsafe {
        let stat_struct_p: *const u8 = &(stat[0]) as *const u8;
        return *core::mem::transmute::<*const u8, *const i64>(stat_struct_p.offset(48));
    }
}

fn stat_syscall(call_id: u32, call_value: u64) -> Option<Stat> {
    let mut stat_struct: Stat = [0; 144];
    unsafe {
        let stat_struct_p: *mut u8 = &mut (stat_struct[0]) as *mut u8;
        if syscall_3_arg(call_id, call_value, stat_struct_p as u64, 0) == 0 {
            return Some(stat_struct);
        }
    }
    None
}
// Didn't want to implement the entire stat header with all its types...
pub fn stat_filesize(path: &str) -> Option<i64> {
    let path_buffer = str_to_zero_padded(&path);
    const SYSCALL_ID: u32 = 4; // 4 is stat, fstat is 5.
    let path_p: *const u8 = &(path_buffer[0]) as *const u8;
    let v = stat_syscall(SYSCALL_ID, path_p as u64)?;
    return Some(get_filesize(&v));
}
pub fn fstat_filesize(fd: u64) -> Option<i64> {
    const SYSCALL_ID: u32 = 5; // 4 is stat, fstat is 5.
    let v = stat_syscall(SYSCALL_ID, fd)?;
    return Some(get_filesize(&v));
}

// syscall 9 is mmap; https://github.com/torvalds/linux/blob/5bfc75d92efd494db37f5c4c173d3639d4772966/arch/x86/kernel/sys_x86_64.c#L89-L97
// https://github.com/torvalds/linux/blob/763978ca67a3d7be3915e2035e2a6c331524c748/mm/mmap.c#L1637-L1642

pub fn brk(desired: u64) -> u64 {
    // https://github.com/torvalds/linux/blob/763978ca67a3d7be3915e2035e2a6c331524c748/mm/mmap.c#L195
    const SYSCALL_ID: u32 = 12;
    unsafe { syscall_1_arg(SYSCALL_ID, desired) }
}

pub fn sbrk(delta: i64) -> *mut u8 {
    let current = brk(0);
    let new = delta + current as i64;
    brk(new as u64) as *mut u8
}

pub mod test {
    use super::*;
    use crate::io::*;
    use crate::println;
    pub fn test_all() {
        test_brk();
        test_file_io();
    }
    pub fn test_brk() {
        let v = sbrk(0);
        println!("0x{:?}", v);
        let x = sbrk(10000);
        println!("0x{:?}", x);
        let x = sbrk(-10000);
        println!("0x{:?}", x);
    }

    pub fn test_file_io() {
        let test_path = "/tmp/barz";
        let f = unsafe { open(test_path, O_CREAT | O_RDWR, 0o0640) };
        println!("{:?}", f);

        assert!(f != 0);

        let test_string = "My awesome test string.\n";
        let path_buffer = str_to_zero_padded(&test_string);
        unsafe {
            let path_p: *const u8 = &(path_buffer[0]) as *const u8;
            let write_res = write(f, path_p, test_string.len() as u64);
            assert_eq!(write_res, test_string.len() as u64);
        }
        let res = unsafe { close(f) };
        assert!(res == 0);

        let v = stat_filesize(test_path);
        assert!(v.is_some());
        let v = v.unwrap();
        assert_eq!(v, test_string.len() as i64);

        let f = unsafe { open(test_path, O_RDONLY, 0) };
        let mut tmp_buffer: [u8; 1024] = [0; 1024];
        let buffer_p: *mut u8 = &mut (tmp_buffer[0]) as *mut u8;
        let res = unsafe { read(f, buffer_p, tmp_buffer.len() as u64) };
        assert_eq!(res, test_string.len() as u64);
        let buffer_ptr = &(tmp_buffer[0]) as *const u8;
        let read_back = unsafe { crate::util::u8_as_str(&buffer_ptr, tmp_buffer.len()) };
        assert_eq!(read_back.unwrap(), test_string);
    }
}
