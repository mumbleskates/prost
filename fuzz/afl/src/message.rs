use afl::fuzz;
use common::test_message;

fn main() {
    fuzz!(|data: &[u8]| {
        test_message(data);
    });
}
