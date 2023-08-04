use std::cell::Cell;

extern "C" {
    fn assert_points_to_something(p: *const i32);
}

fn main() {
    let cell = Cell::new(0);
    unsafe { assert_points_to_something(Cell::as_ptr(&cell)) }
}
