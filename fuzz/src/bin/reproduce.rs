use fuzz::test_messages::TestAllTypes;
use fuzz::roundtrip;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, filename] = args.as_slice() else {
        println!("Usage: {} <path-to-crash>", args[0]);
        std::process::exit(1);
    };

    let data = std::fs::read(&filename).expect(&format!("Could not open file {filename}"));
    let _ = roundtrip::<TestAllTypes>(&data).unwrap_error();
}
