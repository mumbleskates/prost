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

    cd fuzz/afl/
    cargo afl build --bin fuzz-target
    cargo afl fuzz -i in -o out target/debug/fuzz-target

To reproduce a crash, use the `reproduce` binary in the "fuzz" directory:

    cargo run --package fuzz --bin reproduce -- <crashfile>
