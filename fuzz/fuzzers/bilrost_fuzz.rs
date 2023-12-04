#![no_main]

use libfuzzer_sys::fuzz_target;
use test_messages::TestAllTypesProto3;
use tests::roundtrip;

fuzz_target!(|data: &[u8]| {
    let _ = roundtrip::<TestAllTypes>(data).unwrap_error();
});
