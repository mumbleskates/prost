use fuzz::roundtrip;
use fuzz::test_messages::TestAllTypes;

fn main() {
    let mut args = std::env::args();
    let program_name = args.next().unwrap();

    let mut ran = false;
    for filename in args {
        ran = true;
        let data = std::fs::read(&filename).expect(&format!("Could not open file {filename:?}"));
        let _ = roundtrip::<TestAllTypes>(&data).unwrap_error();
    }
    if !ran {
        println!("Usage: {program_name} <path-to-input> [...]");
        std::process::exit(1);
    }
}
