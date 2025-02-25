//@error-pattern: is a dangling pointer
//@compile-flags: -Zmiri-permissive-provenance

fn main() {
    // Can't offset an integer pointer by non-zero offset.
    unsafe {
        let _val = (1 as *mut u8).offset(1);
    }
}
