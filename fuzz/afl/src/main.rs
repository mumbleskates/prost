use afl::fuzz;
use fuzz::test_input;

fn main() {
    fuzz!(|data: &[u8]| {
        test_input(data);
    });
}
