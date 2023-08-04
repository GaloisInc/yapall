use std::any::Any;

extern "C" {
    fn assert_points_to_something(p: *const i32);
}

fn main() {
    let any: Box<dyn Any> = Box::new(0 as i32);
    let ptr = Box::into_raw(any.downcast::<i32>().unwrap());
    unsafe { assert_points_to_something(ptr) }
}
