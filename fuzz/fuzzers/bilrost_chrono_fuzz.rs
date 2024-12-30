#![no_main]

use common::test_chrono_types;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    test_chrono_types(data);
});
