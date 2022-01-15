use crate::io::*;
use crate::println;

pub extern crate core;

use core::mem::transmute;

use crate::support::strlen;

use core::ffi::c_void;


unsafe fn void_as_bytes<'a>(p : &'a *const c_void, limit: usize) -> &'a [u8]
{
    let p = transmute::<*const c_void, *const u8>(*p);
    let len = strlen(p, limit);
    core::slice::from_raw_parts(p, len)
}

unsafe fn void_as_str<'a>(p: &'a *const c_void, limit: usize) -> Result<&'a str, core::str::Utf8Error> 
{
    core::str::from_utf8(void_as_bytes(&p, limit))
}


#[repr(C)]
union aux_entry {
    a_val: i64,
    a_ptr: *const c_void,
    a_fnc: unsafe extern "C" fn(),
}
#[repr(C)]
struct auxv_t {
    a_type: i32,
    a_un: aux_entry,
}

impl core::fmt::Debug for auxv_t
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut w = f.debug_struct("auxv_t");
        unsafe {
        match self.a_type
        {
            0 => w.field("NULL", &0),
            3 => w.field("AT_EXECFD",&format_args!("0x{:x}", &self.a_un.a_val)),
            11 => w.field("AT_UID",&format_args!("0x{:x}", &self.a_un.a_val)),
            9 => w.field("AT_ENTRY", &self.a_un.a_ptr),
            31 => w.field("AT_EXECFN", &format_args!("{:?}", void_as_str(&self.a_un.a_ptr, 1000))),
            15 => w.field("AT_PLATFORM", &format_args!("{:?}", void_as_str(&self.a_un.a_ptr, 1000))),
            24 => w.field("AT_BASE_PLATFORM", &format_args!("{:?}", void_as_str(&self.a_un.a_ptr, 1000))),
            z => w.field("-", &format_args!("{}, 0x{:x}", z, &self.a_un.a_val)),
        };
        }
        w.finish()
    }
}

type ConstCharPtr = *const u8;
type ConstCharPtrArray = *const ConstCharPtr;
type AuxPtr = *const auxv_t;
pub struct AbiContext {
    rsp: *const u8,
    arg_count: usize,
    arg_ptr: ConstCharPtrArray,
    environ_ptr: ConstCharPtrArray,
    environ_count: usize,
    aux_ptr: AuxPtr,
    aux_count: usize,
}

impl AbiContext {
    unsafe fn setup(&mut self) {
        // argc is at the location of rsp.
        self.arg_count = *transmute::<*const u8, *const usize>(self.rsp);

        // arguments are one byte furher.
        self.arg_ptr = transmute::<*const u8, ConstCharPtrArray>(self.rsp.offset(8));

        // environs are a bit further still.
        self.environ_ptr = transmute::<*const u8, ConstCharPtrArray>(
            self.rsp.offset(2 * 8 + 8 * self.arg_count as isize),
        );

        // Determine environ_count:
        while *self.environ_ptr.offset(self.environ_count as isize) != 0 as ConstCharPtr {
            self.environ_count += 1;
        }

        self.aux_ptr = transmute::<*const u8, AuxPtr>(
            self.rsp.offset(3 * 8 + 8 * (self.arg_count + self.environ_count) as isize),
        );
        while (*self.aux_ptr.offset(self.aux_count as isize)).a_type != 0 {
            self.aux_count += 1;
        }
    }

    pub fn new(rsp: *const u8) -> Self {
        let mut v = AbiContext {
            rsp,
            arg_count: 0,
            arg_ptr: 0 as ConstCharPtrArray,
            environ_ptr: 0 as ConstCharPtrArray,
            environ_count: 0,
            aux_ptr: 0 as AuxPtr,
            aux_count: 0,
        };
        unsafe {
            v.setup();
        }
        v
    }

    // Dispatches to execfn.
    pub fn entry(&self) -> !
    {
        unsafe {
            let mut v: isize = 0;
            loop {
                if  (*self.aux_ptr.offset(v)).a_type == 9
                {
                    let addr: i64 = (*self.aux_ptr.offset(v)).a_un.a_val;
                    let orig_rsp = ORIGINAL_RSP;
                    core::arch::asm!("
                        mov ebp, 0 // clear some registers
                        mov r9, 0 // clear some registers
                        mov rdx, 0 // clear some registers
                        push 0 // set the original rsp
                        push {zz} // set the original rsp
                        jmp {entry}", zz = in(reg) orig_rsp, entry = in(reg) addr);
                }
                v += 1;
            }
        }
        loop{}
    }

    fn auxv(&self, index: usize) -> &auxv_t
    {
        unsafe{&*self.aux_ptr.offset(index as isize)}
    }


    pub fn argc(&self) -> usize {
        self.arg_count
    }

    pub fn argv_bytes(&self, index: usize) -> &[u8] {
        if index >= self.argc() {
            panic!("Requested argv beyond argc.");
        }
        unsafe {
            let arg = *self.arg_ptr.offset(index as isize);
            let len = strlen(arg, 1024);
            core::slice::from_raw_parts(arg, len)
        }
    }

    pub fn argv(&self, index: usize) -> Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(self.argv_bytes(index))
    }

    pub fn envc(&self) -> usize {
        self.environ_count
    }

    pub fn env(&self, index: usize) -> Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(self.env_bytes(index))
    }

    pub fn env_bytes(&self, index: usize) -> &[u8] {
        if index >= self.envc() {
            panic!("Requested env beyond envc.");
        }
        unsafe {
            let arg = *self.environ_ptr.offset(index as isize);
            let len = strlen(arg, 8096);
            core::slice::from_raw_parts(arg, len)
        }
    }

    pub fn dump(&self) {
        println!("rsp: {:?}", self.rsp);
        println!("argc: {:?}", self.argc());
        for i in 0..self.argc() {
            // println!("argv_bytes{}: {:?}", i, self.argv_bytes(i));
            println!("argv{}: {:?}", i, self.argv(i));
        }

        println!("envc: {:?}", self.envc());
        for i in 0..self.envc() {
            // println!("argv_bytes{}: {:?}", i, self.argv_bytes(i));
            println!("env{}: {:?}", i, self.env(i));
        }
        println!("self.aux_count: {:?}", self.aux_count);
        for i in 0..self.aux_count {
            // println!("argv_bytes{}: {:?}", i, self.argv_bytes(i));
            println!("auxv{}: {:?}", i, self.auxv(i));
        }
    }
}

static mut ORIGINAL_RSP: u64 = 0;

pub fn context() -> AbiContext {
    return AbiContext::new(unsafe { transmute::<u64, *const u8>(crate::abi::ORIGINAL_RSP) });
}

// state that there will be a main.
extern "C" {
    pub fn main();
}

#[no_mangle]
pub unsafe extern "C" fn _start_stage2() {
    // Rdi was stored from rsp in _start, so here we can really read it, and store it for
    // posterity.
    core::arch::asm!("nop", out("rdi") ORIGINAL_RSP);

    // Then, we can go into main itself.
    main();
}

#[no_mangle]
#[naked] // disable prologue; https://github.com/nox/rust-rfcs/blob/master/text/1201-naked-fns.md
pub unsafe extern "C" fn _start() {
    core::arch::asm!(
        "mov rdi, rsp
    // invoke main.
    call _start_stage2",
        options(noreturn)
    ); // can't read rsp into original_rsp here, as it is zero, or just not used?
}
