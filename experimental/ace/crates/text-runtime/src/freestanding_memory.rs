use core::ffi::c_void;
use core::ptr::{read_volatile, write_volatile};

// Freestanding mem* for the no-CRT PE link: struct initialization and
// by-value moves in this crate lower to memcpy/memset libcalls. Volatile
// byte loops so the compiler cannot pattern-match them back into a call to
// themselves.
#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    let dst = dst.cast::<u8>();
    let src = src.cast::<u8>();
    for i in 0..n {
        write_volatile(dst.add(i), read_volatile(src.add(i)));
    }
    dst.cast()
}

#[no_mangle]
pub unsafe extern "C" fn memmove(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    let dst = dst.cast::<u8>();
    let src = src.cast::<u8>();
    if src < dst as *const u8 && src.add(n) > dst as *const u8 {
        let mut i = n;
        while i > 0 {
            i -= 1;
            write_volatile(dst.add(i), read_volatile(src.add(i)));
        }
    } else {
        for i in 0..n {
            write_volatile(dst.add(i), read_volatile(src.add(i)));
        }
    }
    dst.cast()
}

#[no_mangle]
pub unsafe extern "C" fn memset(dst: *mut c_void, val: i32, n: usize) -> *mut c_void {
    let dst = dst.cast::<u8>();
    let byte = val as u8;
    for i in 0..n {
        write_volatile(dst.add(i), byte);
    }
    dst.cast()
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> i32 {
    let a = a.cast::<u8>();
    let b = b.cast::<u8>();
    for i in 0..n {
        let (x, y) = (read_volatile(a.add(i)), read_volatile(b.add(i)));
        if x != y {
            return x as i32 - y as i32;
        }
    }
    0
}
