fn demo_mut_advanced_unique(mut our: Box<i32>) -> i32 {
    unknown_code_1(&*our);

    // This "re-asserts" uniqueness of the reference: After writing, we know
    // our tag is at the top of the stack.
    *our = 5;

    unknown_code_2();

    // We know this will return 5
    *our
}

// Now comes the evil context
use std::ptr;

static mut LEAK: *mut i32 = ptr::null_mut();

fn unknown_code_1(x: &i32) {
    unsafe {
        LEAK = x as *const _ as *mut _;
    }
}

fn unknown_code_2() {
    unsafe {
        *LEAK = 7; //~ ERROR borrow stack
    }
}

fn main() {
    demo_mut_advanced_unique(Box::new(0));
}
