// Validation makes this fail in the wrong place
// Make sure we find these even with many checks disabled.
//@compile-flags: -Zmiri-disable-alignment-check -Zmiri-disable-stacked-borrows -Zmiri-disable-validation

//@error-pattern: enum value has invalid tag

use std::mem;

#[repr(C)]
pub enum Foo {
    A,
    B,
    C,
    D,
}

fn main() {
    let f = unsafe { std::mem::transmute::<i32, Foo>(42) };
    let _val = mem::discriminant(&f);
}
