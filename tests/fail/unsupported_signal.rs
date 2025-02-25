//! `signal()` is special on Linux and macOS that it's only supported within libstd.
//! The implementation is not complete enough to permit user code to call it.
//@ignore-windows: No libc on Windows
#![feature(rustc_private)]

extern crate libc;

fn main() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_IGN);
        //~^ ERROR unsupported operation: can't call foreign function: signal
    }
}
