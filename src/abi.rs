use crate::io::*;
use crate::println;

pub extern crate core;

use core::mem::transmute;

use crate::support::strlen;

use core::ffi::c_void;

use crate::util::void_as_str;

#[repr(C)]
union aux_entry {
    a_val: i64,
    a_ptr: *const c_void,
    a_fnc: unsafe extern "C" fn(),
}
/// Struct used to encode the auxiliary vector data.
#[repr(C)]
struct auxv_t {
    a_type: i32,
    a_un: aux_entry,
}

impl auxv_t {
    const AT_NULL: i32 = 0;
    const AT_EXECFD: i32 = 2;
    const AT_PHDR: i32 = 3;
    const AT_PHENT: i32 = 4;
    const AT_PHNUM: i32 = 5;
    const AT_ENTRY: i32 = 9;
    const AT_UID: i32 = 11;
    const AT_PLATFORM: i32 = 15;
    const AT_BASE_PLATFORM: i32 = 24;
    const AT_EXECFN: i32 = 31;
}

impl core::fmt::Debug for auxv_t {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut w = f.debug_struct("auxv_t");
        unsafe {
            match self.a_type {
                auxv_t::AT_NULL => w.field("NULL", &0),
                auxv_t::AT_EXECFD => {
                    w.field("AT_EXECFD", &format_args!("0x{:x}", &self.a_un.a_val))
                }
                auxv_t::AT_PHDR => w.field("AT_PHDR", &self.a_un.a_ptr),
                auxv_t::AT_PHENT => w.field("AT_PHENT", &format_args!("0x{:x}", &self.a_un.a_val)),
                auxv_t::AT_PHNUM => w.field("AT_PHNUM", &format_args!("0x{:x}", &self.a_un.a_val)),
                auxv_t::AT_UID => w.field("AT_UID", &format_args!("0x{:x}", &self.a_un.a_val)),
                auxv_t::AT_ENTRY => w.field("AT_ENTRY", &self.a_un.a_ptr),
                auxv_t::AT_EXECFN => w.field(
                    "AT_EXECFN",
                    &format_args!("{:?}", void_as_str(&self.a_un.a_ptr, 1000)),
                ),
                auxv_t::AT_PLATFORM => w.field(
                    "AT_PLATFORM",
                    &format_args!("{:?}", void_as_str(&self.a_un.a_ptr, 1000)),
                ),
                auxv_t::AT_BASE_PLATFORM => w.field(
                    "AT_BASE_PLATFORM",
                    &format_args!("{:?}", void_as_str(&self.a_un.a_ptr, 1000)),
                ),
                z => w.field("-", &format_args!("{}, 0x{:x}", z, &self.a_un.a_val)),
            };
        }
        w.finish()
    }
}

type ConstCharPtr = *const u8;
type ConstCharPtrArray = *const ConstCharPtr;
type AuxPtr = *const auxv_t;
/// Struct to capture the ABI variables and interact with them.
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

        // Grab the auxiliary vectors.
        self.aux_ptr = transmute::<*const u8, AuxPtr>(
            self.rsp
                .offset(3 * 8 + 8 * (self.arg_count + self.environ_count) as isize),
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

    /// Jump to the instruction address with rsp set.
    pub fn jump(address: i64) -> ! {
        unsafe {
            let orig_rsp = ORIGINAL_RSP;
            core::arch::asm!("
                mov rsp, {zz} // set the original rsp
                jmp {entry}", zz = in(reg) orig_rsp, entry = in(reg) address);
        }
        unreachable!();
    }

    /// Dispatches to execfn, stored in at_entry.
    pub fn entry(&self) -> ! {
        unsafe {
            let index = self
                .get_auxv_index(auxv_t::AT_ENTRY)
                .expect("Caller should check have_entry()");
            let addr: i64 = (*self.aux_ptr.offset(index)).a_un.a_val;
            AbiContext::jump(addr);
        }
    }

    /// Obtain the AT_ENTRY index.
    fn get_auxv_index(&self, desired: i32) -> Option<isize> {
        unsafe {
            let mut v: isize = 0;
            loop {
                let a_type = (*self.aux_ptr.offset(v)).a_type;
                if a_type == desired {
                    return Some(v);
                } else if a_type == auxv_t::AT_NULL {
                    break;
                }
                v += 1;
            }
        }
        None
    }

    pub fn get_phdr(&self) -> Option<(*const u8, u64, u64)> {
        let z = self.get_auxv_index(auxv_t::AT_PHDR)?;
        let phdr;
        let phent;
        let phnum;
        unsafe {
            phdr = core::mem::transmute::<_, *const u8>(self.auxv(z).a_un.a_ptr);
            phent = self.auxv(self.get_auxv_index(auxv_t::AT_PHENT)?).a_un.a_val;
            phnum = self.auxv(self.get_auxv_index(auxv_t::AT_PHNUM)?).a_un.a_val;
        }
        Some((phdr, phent as u64, phnum as u64))
    }

    /// Check if we are running as an interpreter.
    pub fn is_interpreter(&self) -> bool {
        unsafe {
            if let Some(index) = self.get_auxv_index(auxv_t::AT_ENTRY) {
                // Check if the AT_ENTRY is not equal to _start
                // if it isn't, we are acting as an interpreter.
                let entry_record = (self.auxv(index as isize)).a_un.a_ptr;
                let start_pointer = _start as *const unsafe extern "C" fn();
                return entry_record != start_pointer as *const c_void;
            }
        }
        false
    }

    /// Get an auxiliary vector entry.
    fn auxv(&self, index: isize) -> &auxv_t {
        unsafe { &*self.aux_ptr.offset(index) }
    }

    /// Get the number of arguments.
    pub fn argc(&self) -> usize {
        self.arg_count
    }

    /// Get an argv entry as bytes.
    pub fn argv_bytes(&self, index: usize) -> &[u8] {
        if index >= self.argc() {
            panic!("Requested argv beyond argc.");
        }
        unsafe {
            let arg = *self.arg_ptr.offset(index as isize);
            let len = strlen(arg, usize::MAX);
            core::slice::from_raw_parts(arg, len)
        }
    }

    /// Get an argv entry as string.
    pub fn argv(&self, index: usize) -> Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(self.argv_bytes(index))
    }

    /// Get the environment variable count.
    pub fn envc(&self) -> usize {
        self.environ_count
    }

    /// Get an environment variable as string.
    pub fn env(&self, index: usize) -> Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(self.env_bytes(index))
    }

    /// Get an environment variable as bytes.
    pub fn env_bytes(&self, index: usize) -> &[u8] {
        if index >= self.envc() {
            panic!("Requested env beyond envc.");
        }
        unsafe {
            let arg = *self.environ_ptr.offset(index as isize);
            let len = strlen(arg, usize::MAX);
            core::slice::from_raw_parts(arg, len)
        }
    }

    /// Dump the execution environment, rsp, all arg, env and aux data.
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
            println!("auxv{}: {:?}", i, self.auxv(i as isize));
        }
    }
}

/// Static variable to store the original RSP back when we first start.
static mut ORIGINAL_RSP: u64 = 0;

/// Function to create a new instance of the context.
pub fn context() -> AbiContext {
    return AbiContext::new(unsafe { transmute::<u64, *const u8>(crate::abi::ORIGINAL_RSP) });
}

/// State that there will be a main() function we'll be starting from our preamble functions below.
extern "C" {
    pub fn main();
}

/// Stage 2 starter, this function is not naked, so we can store RSP for real, and start the main()
/// function.
#[no_mangle]
pub unsafe extern "C" fn _start_stage2() {
    // Rdi was stored from rsp in _start, so here we can really read it, and store it for
    // posterity.
    core::arch::asm!("nop", out("rdi") ORIGINAL_RSP);

    // Ensure memory can do setup, that is allocate the first record.
    crate::mem::setup();

    // Then, we can go into main itself.
    main();
}

/// The entry point of our program, naked function to prevent the prologue, this function copies
/// $rsp into rdx and then calls the stage 2 start function.
/// It also takes care of stack alignment. http://dbp-consulting.com/tutorials/debugging/linuxProgramStartup.html
#[no_mangle]
#[naked] // disable prologue; https://github.com/nox/rust-rfcs/blob/master/text/1201-naked-fns.md
pub unsafe extern "C" fn _start() {
    core::arch::asm!(
        "xor ebp, ebp  // Sets ebp to zero, recommended by ABI.
         xor rbp, rbp  // Lets do the same for the 64 bit register.
         mov rdi, rsp // Store the original rsp.
         // Then, we can just enforce stack alignment.
         and rsp, 0xFFFFFFFFfffffff0 // enforce stack alignment on 16 bytes.
        // invoke the second stage start method with the aligned stack.
        call _start_stage2",
        options(noreturn)
    ); // can't read rsp into original_rsp here, may not be allowed in naked functions?
}
