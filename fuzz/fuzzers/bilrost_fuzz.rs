#![no_main]

use libfuzzer_sys::fuzz_target;
use fuzz::test_messages::TestAllTypes;
use fuzz::roundtrip;

fuzz_target!(|data: &[u8]| {
    let _ = roundtrip::<TestAllTypes>(data).unwrap_error();
});
