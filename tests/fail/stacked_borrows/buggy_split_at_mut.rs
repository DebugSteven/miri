mod safe {
    use std::slice::from_raw_parts_mut;

    pub fn split_at_mut<T>(self_: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
        let len = self_.len();
        let ptr = self_.as_mut_ptr();

        unsafe {
            assert!(mid <= len);

            (
                from_raw_parts_mut(ptr, len - mid), // BUG: should be "mid" instead of "len - mid"
                from_raw_parts_mut(ptr.offset(mid as isize), len - mid),
            )
        }
    }
}

fn main() {
    let mut array = [1, 2, 3, 4];
    let (a, b) = safe::split_at_mut(&mut array, 0);
    //~^ ERROR borrow stack
    a[1] = 5;
    b[1] = 6;
}
