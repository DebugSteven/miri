//@ignore-windows: No libc on Windows

#![feature(rustc_private)]

extern crate libc;

fn main() {
    unsafe {
        let mut mutexattr: libc::pthread_mutexattr_t = std::mem::zeroed();
        assert_eq!(
            libc::pthread_mutexattr_settype(&mut mutexattr as *mut _, libc::PTHREAD_MUTEX_NORMAL),
            0,
        );
        let mut mutex: libc::pthread_mutex_t = std::mem::zeroed();
        assert_eq!(libc::pthread_mutex_init(&mut mutex as *mut _, &mutexattr as *const _), 0);
        assert_eq!(libc::pthread_mutex_lock(&mut mutex as *mut _), 0);
        assert_eq!(libc::pthread_mutex_unlock(&mut mutex as *mut _), 0);
        libc::pthread_mutex_unlock(&mut mutex as *mut _); //~ ERROR was not locked
    }
}
