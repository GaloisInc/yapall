extern "C" {
    fn assert_points_to_something(p: *const &i32);
    // fn assert_may_alias(p: *const i32, q: *const i32);
}

fn main() {
    let x = 0;
    let mut v = Vec::new();
    v.push(&x);
    unsafe { assert_points_to_something(Vec::as_ptr(&v)) }
    // TODO: This fails.
    // unsafe { assert_may_alias(&x, *Vec::as_ptr(&v)) }
}
