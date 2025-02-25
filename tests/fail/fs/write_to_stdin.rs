//@ignore-windows: No libc on Windows

#![feature(rustc_private)]

extern crate libc;

fn main() -> std::io::Result<()> {
    let bytes = b"hello";
    unsafe {
        libc::write(0, bytes.as_ptr() as *const libc::c_void, 5); //~ ERROR cannot write to stdin
    }
    Ok(())
}
