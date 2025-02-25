//@ignore-windows: No libc on Windows

#![feature(rustc_private)]

extern crate libc;

fn main() -> std::io::Result<()> {
    let mut bytes = [0u8; 512];
    unsafe {
        libc::read(0, bytes.as_mut_ptr() as *mut libc::c_void, 512); //~ ERROR `read` from stdin not available when isolation is enabled
    }
    Ok(())
}
