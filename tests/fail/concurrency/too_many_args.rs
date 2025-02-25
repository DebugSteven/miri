//@ignore-windows: Concurrency on Windows is not supported yet.

//! The thread function must have exactly one argument.

#![feature(rustc_private)]

extern crate libc;

use std::{mem, ptr};

extern "C" fn thread_start(_null: *mut libc::c_void, _x: i32) -> *mut libc::c_void {
    panic!() //~ ERROR: callee has more arguments than expected
}

fn main() {
    unsafe {
        let mut native: libc::pthread_t = mem::zeroed();
        let attr: libc::pthread_attr_t = mem::zeroed();
        // assert_eq!(libc::pthread_attr_init(&mut attr), 0); FIXME: this function is not yet implemented.
        let thread_start: extern "C" fn(*mut libc::c_void, i32) -> *mut libc::c_void = thread_start;
        let thread_start: extern "C" fn(*mut libc::c_void) -> *mut libc::c_void =
            mem::transmute(thread_start);
        assert_eq!(libc::pthread_create(&mut native, &attr, thread_start, ptr::null_mut()), 0);
        assert_eq!(libc::pthread_join(native, ptr::null_mut()), 0);
    }
}
