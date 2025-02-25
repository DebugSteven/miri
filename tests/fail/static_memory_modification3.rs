// Stacked Borrows detects that we are casting & to &mut and so it changes why we fail
//@compile-flags: -Zmiri-disable-stacked-borrows

use std::mem::transmute;

#[allow(mutable_transmutes)]
fn main() {
    unsafe {
        let bs = b"this is a test";
        transmute::<&[u8], &mut [u8]>(bs)[4] = 42; //~ ERROR read-only
    }
}
