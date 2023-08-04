use std::sync::Arc;

extern "C" {
    fn assert_points_to_something(p: *const i32);
}

fn main() {
    let arc = Arc::new(0);
    unsafe { assert_points_to_something(Arc::as_ptr(&arc)) }
}
