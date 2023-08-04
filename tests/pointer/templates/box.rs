extern "C" {
    fn assert_points_to_something(p: *const i32);
}

fn main() {
    let b = Box::new(0);
    unsafe { assert_points_to_something(Box::into_raw(b)) }
}
