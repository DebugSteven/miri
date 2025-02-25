//@compile-flags: -Zmiri-permissive-provenance
#![feature(strict_provenance)]

// If we have only exposed read-only pointers, doing a write through a wildcard ptr should fail.

fn main() {
    let mut x = 0;
    let _fool = &mut x as *mut i32; // this would have fooled the old untagged pointer logic
    let addr = (&x as *const i32).expose_addr();
    let ptr = std::ptr::from_exposed_addr_mut::<i32>(addr);
    unsafe { *ptr = 0 }; //~ ERROR: borrow stack
}
