extern "C" {
    fn assert_points_to_something(p: *const &i32);
    fn assert_may_alias(p: *const i32, q: *const i32);
}

fn main() {
    let i = 1;
    let j = 2;
    let x = &[&i, &j];
    let x_ptr = x.as_ptr();
    unsafe { assert_points_to_something(x_ptr) }
    unsafe { assert_may_alias(*x_ptr, &i) }
}
