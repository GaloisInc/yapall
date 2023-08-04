use std::rc::Rc;

extern "C" {
    fn assert_points_to_something(p: *const i32);
}

fn main() {
    let rc = Rc::new(0);
    unsafe { assert_points_to_something(Rc::as_ptr(&rc)) }
}
