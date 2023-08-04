extern "C" {
    fn assert_reachable();
}

fn main() {
    unsafe { assert_reachable() }
}
