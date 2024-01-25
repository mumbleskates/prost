use afl::fuzz;
use anyhow::anyhow;
use fuzz::roundtrip;
use fuzz::test_messages::TestAllTypes;

fn main() {
    fuzz!(|data: &[u8]| {
        let _ = roundtrip::<TestAllTypes>(data).unwrap_error();
    });
}
