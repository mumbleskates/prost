#![no_main]

use fuzz::test_messages::{TestAllTypes, TestDistinguished};
use fuzz::{roundtrip, roundtrip_distinguished};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = roundtrip::<TestAllTypes>(data).unwrap_error();
    let _ = roundtrip_distinguished::<TestDistinguished>(data).unwrap_error();
});
