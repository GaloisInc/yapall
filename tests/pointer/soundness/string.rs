extern "C" {
    fn assert_points_to_something(p: *const u8);
}

fn main() {
    let p = String::new().as_ptr();
    unsafe { assert_points_to_something(p) }
}
