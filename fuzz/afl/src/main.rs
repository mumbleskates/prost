use afl::fuzz;
use common::test_input;

fn main() {
    fuzz!(|data: &[u8]| {
        test_input(data);
    });
}
