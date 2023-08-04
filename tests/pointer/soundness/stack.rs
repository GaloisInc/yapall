extern "C" {
    fn assert_points_to_something(p: *const i32);
}

fn main() {
    unsafe { assert_points_to_something(&5) }
}
