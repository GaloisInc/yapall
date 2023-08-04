extern "C" {
    fn assert_points_to_something(p: *const u8);
}

fn main() {
    unsafe { assert_points_to_something("".as_ptr()) }
}
