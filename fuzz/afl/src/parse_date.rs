use afl::fuzz;
use common::test_parse_date;

fn main() {
    fuzz!(|data: &[u8]| {
        test_parse_date(data);
    });
}
