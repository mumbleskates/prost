# Fuzzing

Bilrost ships a few fuzz tests, using both libfuzzer and aflfuzz.

To run the libfuzzer tests, first install cargo-fuzz:

    cargo install cargo-fuzz

Then the fuzzer can be run:

    cargo fuzz run bilrost_fuzz -- <flags>

See [the libfuzzer docs](https://llvm.org/docs/LibFuzzer.html) for options and
further info.

To run the afl fuzz tests, first install cargo-afl:

    cargo install cargo-afl

Then build a fuzz target and run afl on it:

    cd fuzz
    cargo afl build --package fuzz-afl --bin bilrost_afl
    cargo afl fuzz -i afl/in -o afl/out target/debug/bilrost_afl

To reproduce a crash, use the `reproduce` binary in the "fuzz/common" directory:

    cd fuzz
    cargo run --package common --bin reproduce -- <crashfile>
