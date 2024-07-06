#![no_main]

use common::test_parse_date;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    test_parse_date(data);
});
