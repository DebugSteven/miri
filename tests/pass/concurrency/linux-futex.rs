//@only-linux
//@compile-flags: -Zmiri-disable-isolation

#![feature(rustc_private)]
extern crate libc;

use std::mem::MaybeUninit;
use std::ptr;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

fn wake_nobody() {
    let futex = 0;

    // Wake 1 waiter. Expect zero waiters woken up, as nobody is waiting.
    unsafe {
        assert_eq!(libc::syscall(libc::SYS_futex, &futex as *const i32, libc::FUTEX_WAKE, 1), 0);
    }

    // Same, but without omitting the unused arguments.
    unsafe {
        assert_eq!(
            libc::syscall(
                libc::SYS_futex,
                &futex as *const i32,
                libc::FUTEX_WAKE,
                1,
                ptr::null::<libc::timespec>(),
                0usize,
                0,
            ),
            0,
        );
    }
}

fn wake_dangling() {
    let futex = Box::new(0);
    let ptr: *const i32 = &*futex;
    drop(futex);

    // Wake 1 waiter. Expect zero waiters woken up, as nobody is waiting.
    unsafe {
        assert_eq!(libc::syscall(libc::SYS_futex, ptr, libc::FUTEX_WAKE, 1), 0);
    }
}

fn wait_wrong_val() {
    let futex: i32 = 123;

    // Only wait if the futex value is 456.
    unsafe {
        assert_eq!(
            libc::syscall(
                libc::SYS_futex,
                &futex as *const i32,
                libc::FUTEX_WAIT,
                456,
                ptr::null::<libc::timespec>(),
            ),
            -1,
        );
        assert_eq!(*libc::__errno_location(), libc::EAGAIN);
    }
}

fn wait_timeout() {
    let start = Instant::now();

    let futex: i32 = 123;

    // Wait for 200ms, with nobody waking us up early.
    unsafe {
        assert_eq!(
            libc::syscall(
                libc::SYS_futex,
                &futex as *const i32,
                libc::FUTEX_WAIT,
                123,
                &libc::timespec { tv_sec: 0, tv_nsec: 200_000_000 },
            ),
            -1,
        );
        assert_eq!(*libc::__errno_location(), libc::ETIMEDOUT);
    }

    assert!((200..1000).contains(&start.elapsed().as_millis()));
}

fn wait_absolute_timeout() {
    let start = Instant::now();

    // Get the current monotonic timestamp as timespec.
    let mut timeout = unsafe {
        let mut now: MaybeUninit<libc::timespec> = MaybeUninit::uninit();
        assert_eq!(libc::clock_gettime(libc::CLOCK_MONOTONIC, now.as_mut_ptr()), 0);
        now.assume_init()
    };

    // Add 200ms.
    timeout.tv_nsec += 200_000_000;
    if timeout.tv_nsec > 1_000_000_000 {
        timeout.tv_nsec -= 1_000_000_000;
        timeout.tv_sec += 1;
    }

    let futex: i32 = 123;

    // Wait for 200ms from now, with nobody waking us up early.
    unsafe {
        assert_eq!(
            libc::syscall(
                libc::SYS_futex,
                &futex as *const i32,
                libc::FUTEX_WAIT_BITSET,
                123,
                &timeout,
                0usize,
                u32::MAX,
            ),
            -1,
        );
        assert_eq!(*libc::__errno_location(), libc::ETIMEDOUT);
    }

    assert!((200..1000).contains(&start.elapsed().as_millis()));
}

fn wait_wake() {
    let start = Instant::now();

    static FUTEX: i32 = 0;

    let t = thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        unsafe {
            assert_eq!(
                libc::syscall(
                    libc::SYS_futex,
                    &FUTEX as *const i32,
                    libc::FUTEX_WAKE,
                    10, // Wake up at most 10 threads.
                ),
                1, // Woken up one thread.
            );
        }
    });

    unsafe {
        assert_eq!(
            libc::syscall(
                libc::SYS_futex,
                &FUTEX as *const i32,
                libc::FUTEX_WAIT,
                0,
                ptr::null::<libc::timespec>(),
            ),
            0,
        );
    }

    assert!((200..1000).contains(&start.elapsed().as_millis()));
    t.join().unwrap();
}

fn wait_wake_bitset() {
    let start = Instant::now();

    static FUTEX: i32 = 0;

    let t = thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        unsafe {
            assert_eq!(
                libc::syscall(
                    libc::SYS_futex,
                    &FUTEX as *const i32,
                    libc::FUTEX_WAKE_BITSET,
                    10, // Wake up at most 10 threads.
                    ptr::null::<libc::timespec>(),
                    0usize,
                    0b1001, // bitset
                ),
                0, // Didn't match any thread.
            );
        }
        thread::sleep(Duration::from_millis(200));
        unsafe {
            assert_eq!(
                libc::syscall(
                    libc::SYS_futex,
                    &FUTEX as *const i32,
                    libc::FUTEX_WAKE_BITSET,
                    10, // Wake up at most 10 threads.
                    ptr::null::<libc::timespec>(),
                    0usize,
                    0b0110, // bitset
                ),
                1, // Woken up one thread.
            );
        }
    });

    unsafe {
        assert_eq!(
            libc::syscall(
                libc::SYS_futex,
                &FUTEX as *const i32,
                libc::FUTEX_WAIT_BITSET,
                0,
                ptr::null::<libc::timespec>(),
                0usize,
                0b0100, // bitset
            ),
            0,
        );
    }

    assert!((400..1000).contains(&start.elapsed().as_millis()));
    t.join().unwrap();
}

fn concurrent_wait_wake() {
    const FREE: i32 = 0;
    const HELD: i32 = 1;

    static FUTEX: AtomicI32 = AtomicI32::new(0);
    static mut DATA: i32 = 0;
    static WOKEN: AtomicI32 = AtomicI32::new(0);

    let rounds = 50;
    for _ in 0..rounds {
        unsafe { DATA = 0 }; // Reset
        // Suppose the main thread is holding a lock implemented using futex...
        FUTEX.store(HELD, Ordering::Relaxed);

        let t = thread::spawn(move || {
            // If this syscall runs first, then we'll be woken up by
            // the main thread's FUTEX_WAKE, and all is fine.
            //
            // If this sycall runs after the main thread's store
            // and FUTEX_WAKE, the syscall must observe that
            // the FUTEX is FREE != HELD and return without waiting
            // or we'll deadlock.
            unsafe {
                let ret = libc::syscall(
                    libc::SYS_futex,
                    &FUTEX as *const AtomicI32,
                    libc::FUTEX_WAIT,
                    HELD,
                    ptr::null::<libc::timespec>(),
                );
                if ret == 0 {
                    // We actually slept. And then woke up again. So we should be ordered-after
                    // what happened-before the FUTEX_WAKE. So this is not a race.
                    assert_eq!(DATA, 1);
                    // Also remember that this happened at least once.
                    WOKEN.fetch_add(1, Ordering::Relaxed);
                }
            }
        });
        // Increase the chance that the other thread actually goes to sleep.
        // (5 yields in a loop seem to make that happen around 40% of the time.)
        for _ in 0..5 {
            thread::yield_now();
        }

        FUTEX.store(FREE, Ordering::Relaxed);
        unsafe {
            DATA = 1;
            libc::syscall(libc::SYS_futex, &FUTEX as *const AtomicI32, libc::FUTEX_WAKE, 1);
        }

        t.join().unwrap();
    }

    // Make sure we got the interesting case (of having woken a thread) at least once, but not *each* time.
    let woken = WOKEN.load(Ordering::Relaxed);
    assert!(woken > 0 && woken < rounds);
    //eprintln!("waking happened {woken} times");
}

fn main() {
    wake_nobody();
    wake_dangling();
    wait_wrong_val();
    wait_timeout();
    wait_absolute_timeout();
    wait_wake();
    wait_wake_bitset();
    concurrent_wait_wake();
}
