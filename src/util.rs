use core::ffi::c_void;
use core::mem::transmute;
use support::strlen;

/// Convert a void pointer with length to a byte slice.
pub unsafe fn void_as_bytes<'a>(p: &'a *const c_void, limit: usize) -> &'a [u8] {
    let p = transmute::<*const c_void, *const u8>(*p);
    let len = strlen(p, limit);
    core::slice::from_raw_parts(p, len)
}
/// Convert a void pointer with maximum string length limit to a proper string.
pub unsafe fn void_as_str<'a>(
    p: &'a *const c_void,
    limit: usize,
) -> Result<&'a str, core::str::Utf8Error> {
    core::str::from_utf8(void_as_bytes(&p, limit))
}
pub unsafe fn u8_as_str<'a>(
    p: &'a *const u8,
    limit: usize,
) -> Result<&'a str, core::str::Utf8Error> {
    let p = transmute::<&*const u8, &*const c_void>(&p);
    core::str::from_utf8(void_as_bytes(&p, limit))
}
