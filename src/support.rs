// Well, format just assumes that memset and memcpy exist.
// We can definitely provide those to make the linker happy, they don't require allocations.

// They're written without `for i in 0..size` to avoid creating an iterator object, at some point
// I was actually stepping through these functions to figure out a segfault.

#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, size: usize) -> *mut u8 {
    let mut i = 0;
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
        if *s1.offset(i as isize) != *s2.offset(i as isize) {
            return if *s1.offset(i as isize) < *s2.offset(i as isize) {
                -1
            } else {
                1
            };
        }
        i += 1;
    }
    0
}
