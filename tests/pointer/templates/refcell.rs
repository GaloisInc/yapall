use std::cell::RefCell;

extern "C" {
    fn assert_points_to_something(p: *const i32);
}

fn main() {
    let refcell = RefCell::new(0);
    unsafe { assert_points_to_something(RefCell::as_ptr(&refcell)) }
}
