#![no_main]

use fuzz::test_input;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    test_input(data);
});
