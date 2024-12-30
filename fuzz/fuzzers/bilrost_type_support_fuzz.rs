#![no_main]

use common::test_type_support;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    test_type_support(data);
});
