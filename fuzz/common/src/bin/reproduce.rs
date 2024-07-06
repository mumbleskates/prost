use bilrost::encoding::opaque::OpaqueMessage;
use bilrost::{DistinguishedMessage, Message};
use common::test_messages::{TestAllTypes, TestDistinguished};

fn main() {
    let mut args = std::env::args();
    let program_name = args.next().unwrap();

    let mut ran = false;
    for filename in args {
        ran = true;
        let data =
            std::fs::read(&filename).unwrap_or_else(|_| panic!("Could not open file {filename:?}"));
        println!("file: {filename:?}");
        println!("opaque: {:#?}", OpaqueMessage::decode(data.as_slice()));
        println!("TestAllTypes: {:#?}", TestAllTypes::decode(data.as_slice()));
        println!(
            "TestDistinguished: {:#?}",
            TestDistinguished::decode_distinguished(data.as_slice())
        );
        common::test_message(&data);
    }
    if !ran {
        println!("Usage: {program_name} <path-to-input> [...]");
        std::process::exit(1);
    }
}
