#![no_main]

use common::test_input;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    test_input(data);
});