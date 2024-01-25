use bilrost::alloc::boxed::Box;
use bilrost::alloc::collections::BTreeMap;
use bilrost::alloc::string::String;
use bilrost::alloc::vec::Vec;
use bilrost::{Enumeration, Message, Oneof};
use core::option::Option;

/// This proto includes every type of field in both singular and repeated
/// forms.
///
/// Also, crucially, all messages and enums in this file are eventually
/// submessages of this message.  So for example, a fuzz test of TestAllTypes
/// could trigger bugs that occur in any message type in this file.  We verify
/// this stays true in a unit test.
#[derive(Clone, PartialEq, Message)]
pub struct TestAllTypes {
    /// Singular
    pub sint32: i32,
    pub sint64: i64,
    pub uint32: u32,
    pub uint64: u64,
    #[bilrost(encoder(fixed))]
    pub ufixed32: u32,
    #[bilrost(encoder(fixed))]
    pub ufixed64: u64,
    #[bilrost(encoder(fixed))]
    pub sfixed32: i32,
    #[bilrost(encoder(fixed))]
    pub sfixed64: i64,
    pub float32: f32,
    pub float64: f64,
    pub bool: bool,
    pub string: String,
    #[bilrost(encoder(vecblob))]
    pub bytes: Vec<u8>,
    pub direct_message: test_message::NestedMessage,
    pub boxed_message: Box<test_message::NestedMessage>,
    #[bilrost(enumeration(test_message::NestedEnum))]
    pub helped_enum: u32,
    pub direct_enum: test_message::NestedEnum,
    pub map_sint32_sint32: BTreeMap<i32, i32>,
    pub map_sint64_sint64: BTreeMap<i64, i64>,
    pub map_uint32_uint32: BTreeMap<u32, u32>,
    pub map_uint64_uint64: BTreeMap<u64, u64>,
    #[bilrost(encoder(map<fixed, fixed>))]
    pub map_ufixed32_ufixed32: BTreeMap<u32, u32>,
    #[bilrost(encoder(map<fixed, fixed>))]
    pub map_ufixed64_ufixed64: BTreeMap<u64, u64>,
    #[bilrost(encoder(map<fixed, fixed>))]
    pub map_sfixed32_sfixed32: BTreeMap<i32, i32>,
    #[bilrost(encoder(map<fixed, fixed>))]
    pub map_sfixed64_sfixed64: BTreeMap<i64, i64>,
    pub map_sint32_float32: BTreeMap<i32, f32>,
    pub map_sint32_float64: BTreeMap<i32, f64>,
    pub map_bool_bool: BTreeMap<bool, bool>,
    pub map_string_string: BTreeMap<String, String>,
    #[bilrost(encoder(map<general, vecblob>))]
    pub map_string_bytes: BTreeMap<String, Vec<u8>>,
    pub map_string_nested_message: BTreeMap<String, test_message::NestedMessage>,
    pub map_string_nested_enum: BTreeMap<String, test_message::NestedEnum>,
    /// Optional
    pub optional_sint32: Option<i32>,
    pub optional_sint64: Option<i64>,
    pub optional_uint32: Option<u32>,
    pub optional_uint64: Option<u64>,
    #[bilrost(encoder(fixed))]
    pub optional_ufixed32: Option<u32>,
    #[bilrost(encoder(fixed))]
    pub optional_ufixed64: Option<u64>,
    #[bilrost(encoder(fixed))]
    pub optional_sfixed32: Option<i32>,
    #[bilrost(encoder(fixed))]
    pub optional_sfixed64: Option<i64>,
    pub optional_float32: Option<f32>,
    pub optional_float64: Option<f64>,
    pub optional_bool: Option<bool>,
    pub optional_string: Option<String>,
    #[bilrost(encoder(vecblob))]
    pub optional_bytes: Option<Vec<u8>>,
    pub optional_direct_message: Option<test_message::NestedMessage>,
    pub optional_boxed_message: Option<Box<test_message::NestedMessage>>,
    #[bilrost(enumeration(test_message::NestedEnum))]
    pub optional_helped_enum: Option<u32>,
    pub optional_direct_enum: Option<test_message::NestedEnum>,
    pub optional_map_sint32_sint32: Option<BTreeMap<i32, i32>>,
    pub optional_map_sint64_sint64: Option<BTreeMap<i64, i64>>,
    pub optional_map_uint32_uint32: Option<BTreeMap<u32, u32>>,
    pub optional_map_uint64_uint64: Option<BTreeMap<u64, u64>>,
    #[bilrost(encoder(map<fixed, fixed>))]
    pub optional_map_ufixed32_ufixed32: Option<BTreeMap<u32, u32>>,
    #[bilrost(encoder(map<fixed, fixed>))]
    pub optional_map_ufixed64_ufixed64: Option<BTreeMap<u64, u64>>,
    #[bilrost(encoder(map<fixed, fixed>))]
    pub optional_map_sfixed32_sfixed32: Option<BTreeMap<i32, i32>>,
    #[bilrost(encoder(map<fixed, fixed>))]
    pub optional_map_sfixed64_sfixed64: Option<BTreeMap<i64, i64>>,
    pub optional_map_sint32_float32: Option<BTreeMap<i32, f32>>,
    pub optional_map_sint32_float64: Option<BTreeMap<i32, f64>>,
    pub optional_map_bool_bool: Option<BTreeMap<bool, bool>>,
    pub optional_map_string_string: Option<BTreeMap<String, String>>,
    #[bilrost(encoder(map<general, vecblob>))]
    pub optional_map_string_bytes: Option<BTreeMap<String, Vec<u8>>>,
    pub optional_map_string_nested_message: Option<BTreeMap<String, test_message::NestedMessage>>,
    pub optional_map_string_nested_enum: Option<BTreeMap<String, test_message::NestedEnum>>,
    /// Unpacked
    pub unpacked_sint32: Vec<i32>,
    pub unpacked_sint64: Vec<i64>,
    pub unpacked_uint32: Vec<u32>,
    pub unpacked_uint64: Vec<u64>,
    #[bilrost(encoder(fixed))]
    pub unpacked_ufixed32: Vec<u32>,
    #[bilrost(encoder(fixed))]
    pub unpacked_ufixed64: Vec<u64>,
    #[bilrost(encoder(fixed))]
    pub unpacked_sfixed32: Vec<i32>,
    #[bilrost(encoder(fixed))]
    pub unpacked_sfixed64: Vec<i64>,
    pub unpacked_float32: Vec<f32>,
    pub unpacked_float64: Vec<f64>,
    pub unpacked_bool: Vec<bool>,
    pub unpacked_string: Vec<String>,
    #[bilrost(encoder(unpacked<vecblob>))]
    pub unpacked_bytes: Vec<Vec<u8>>,
    pub unpacked_nested_message: Vec<test_message::NestedMessage>,
    pub unpacked_map_sint32_sint32: Vec<BTreeMap<i32, i32>>,
    pub unpacked_map_sint64_sint64: Vec<BTreeMap<i64, i64>>,
    pub unpacked_map_uint32_uint32: Vec<BTreeMap<u32, u32>>,
    pub unpacked_map_uint64_uint64: Vec<BTreeMap<u64, u64>>,
    #[bilrost(encoder(unpacked<map<fixed, fixed>>))]
    pub unpacked_map_ufixed32_ufixed32: Vec<BTreeMap<u32, u32>>,
    #[bilrost(encoder(unpacked<map<fixed, fixed>>))]
    pub unpacked_map_ufixed64_ufixed64: Vec<BTreeMap<u64, u64>>,
    #[bilrost(encoder(unpacked<map<fixed, fixed>>))]
    pub unpacked_map_sfixed32_sfixed32: Vec<BTreeMap<i32, i32>>,
    #[bilrost(encoder(unpacked<map<fixed, fixed>>))]
    pub unpacked_map_sfixed64_sfixed64: Vec<BTreeMap<i64, i64>>,
    pub unpacked_map_sint32_float32: Vec<BTreeMap<i32, f32>>,
    pub unpacked_map_sint32_float64: Vec<BTreeMap<i32, f64>>,
    pub unpacked_map_bool_bool: Vec<BTreeMap<bool, bool>>,
    pub unpacked_map_string_string: Vec<BTreeMap<String, String>>,
    #[bilrost(encoder(unpacked<map<general, vecblob>>))]
    pub unpacked_map_string_bytes: Vec<BTreeMap<String, Vec<u8>>>,
    pub unpacked_map_string_nested_message: Vec<BTreeMap<String, test_message::NestedMessage>>,
    pub unpacked_map_string_nested_enum: Vec<BTreeMap<String, test_message::NestedEnum>>,
    /// Packed
    #[bilrost(encoder(packed))]
    pub packed_sint32: Vec<i32>,
    #[bilrost(encoder(packed))]
    pub packed_sint64: Vec<i64>,
    #[bilrost(encoder(packed))]
    pub packed_uint32: Vec<u32>,
    #[bilrost(encoder(packed))]
    pub packed_uint64: Vec<u64>,
    #[bilrost(encoder(packed<fixed>))]
    pub packed_ufixed32: Vec<u32>,
    #[bilrost(encoder(packed<fixed>))]
    pub packed_ufixed64: Vec<u64>,
    #[bilrost(encoder(packed<fixed>))]
    pub packed_sfixed32: Vec<i32>,
    #[bilrost(encoder(packed<fixed>))]
    pub packed_sfixed64: Vec<i64>,
    #[bilrost(encoder(packed))]
    pub packed_float32: Vec<f32>,
    #[bilrost(encoder(packed))]
    pub packed_float64: Vec<f64>,
    #[bilrost(encoder(packed))]
    pub packed_bool: Vec<bool>,
    #[bilrost(encoder(packed))]
    pub packed_nested_enum: Vec<test_message::NestedEnum>,
    #[bilrost(encoder(packed))]
    pub packed_map_sint32_sint32: Vec<BTreeMap<i32, i32>>,
    #[bilrost(encoder(packed))]
    pub packed_map_sint64_sint64: Vec<BTreeMap<i64, i64>>,
    #[bilrost(encoder(packed))]
    pub packed_map_uint32_uint32: Vec<BTreeMap<u32, u32>>,
    #[bilrost(encoder(packed))]
    pub packed_map_uint64_uint64: Vec<BTreeMap<u64, u64>>,
    #[bilrost(encoder(packed<map<fixed, fixed>>))]
    pub packed_map_ufixed32_ufixed32: Vec<BTreeMap<u32, u32>>,
    #[bilrost(encoder(packed<map<fixed, fixed>>))]
    pub packed_map_ufixed64_ufixed64: Vec<BTreeMap<u64, u64>>,
    #[bilrost(encoder(packed<map<fixed, fixed>>))]
    pub packed_map_sfixed32_sfixed32: Vec<BTreeMap<i32, i32>>,
    #[bilrost(encoder(packed<map<fixed, fixed>>))]
    pub packed_map_sfixed64_sfixed64: Vec<BTreeMap<i64, i64>>,
    #[bilrost(encoder(packed))]
    pub packed_map_sint32_float32: Vec<BTreeMap<i32, f32>>,
    #[bilrost(encoder(packed))]
    pub packed_map_sint32_float64: Vec<BTreeMap<i32, f64>>,
    #[bilrost(encoder(packed))]
    pub packed_map_bool_bool: Vec<BTreeMap<bool, bool>>,
    #[bilrost(encoder(packed))]
    pub packed_map_string_string: Vec<BTreeMap<String, String>>,
    #[bilrost(encoder(packed<map<general, vecblob>>))]
    pub packed_map_string_bytes: Vec<BTreeMap<String, Vec<u8>>>,
    #[bilrost(encoder(packed))]
    pub packed_map_string_nested_message: Vec<BTreeMap<String, test_message::NestedMessage>>,
    #[bilrost(encoder(packed))]
    pub packed_map_string_nested_enum: Vec<BTreeMap<String, test_message::NestedEnum>>,
    /// Recursive message
    // pub recursive_message: Option<Box<TestAllTypes>>, // TODO(widders): avoid recursive trait bounds
    /// Well-known types
    #[bilrost(tag = 301)]
    pub direct_duration: bilrost_types::Duration,
    pub direct_timestamp: bilrost_types::Timestamp,
    pub direct_struct: bilrost_types::Struct,
    pub direct_value: bilrost_types::Value,
    pub optional_duration: Option<bilrost_types::Duration>,
    pub optional_timestamp: Option<bilrost_types::Timestamp>,
    pub optional_struct: Option<bilrost_types::Struct>,
    pub optional_value: Option<bilrost_types::Value>,
    pub unpacked_duration: Vec<bilrost_types::Duration>,
    pub unpacked_timestamp: Vec<bilrost_types::Timestamp>,
    pub unpacked_struct: Vec<bilrost_types::Struct>,
    pub unpacked_value: Vec<bilrost_types::Value>,
    pub unpacked_list_value: Vec<bilrost_types::ListValue>,
    #[bilrost(encoder(packed))]
    pub packed_duration: Vec<bilrost_types::Duration>,
    #[bilrost(encoder(packed))]
    pub packed_timestamp: Vec<bilrost_types::Timestamp>,
    #[bilrost(encoder(packed))]
    pub packed_struct: Vec<bilrost_types::Struct>,
    #[bilrost(encoder(packed))]
    pub packed_value: Vec<bilrost_types::Value>,
    #[bilrost(encoder(packed))]
    pub packed_list_value: Vec<bilrost_types::ListValue>,
    /// Oneofs
    #[bilrost(oneof(1001, 1002, 1003, 1004, 1005, 1006, 1007, 1008, 1009))]
    pub oneof_field: Option<test_message::NonEmptyOneofField>,
    #[bilrost(oneof(2001, 2002, 2003, 2004, 2005, 2006, 2007, 2008, 2009))]
    pub nonempty_oneof_field: test_message::OneofField,
}

/// Nested message and enum types in `TestAllTypes`.
pub mod test_message {
    use super::*;

    #[derive(Clone, PartialEq, Message)]
    pub struct NestedMessage {
        pub a: i32,
        // pub corecursive: Option<Box<super::TestAllTypes>>, // TODO(widders): avoid recursive trait bounds
    }
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Enumeration)]
    #[repr(u32)]
    pub enum NestedEnum {
        #[default]
        Foo = 0,
        Bar = 1,
        Baz = 2,
        Max = u32::MAX,
    }
    #[derive(Clone, PartialEq, Oneof)]
    pub enum NonEmptyOneofField {
        #[bilrost(tag = 1001)]
        OneofUint32(u32),
        #[bilrost(tag = 1002)]
        OneofNestedMessage(Box<NestedMessage>),
        #[bilrost(tag = 1003)]
        OneofString(String),
        #[bilrost(tag = 1004, encoder(vecblob))]
        OneofBytes(Vec<u8>),
        #[bilrost(tag = 1005)]
        OneofBool(bool),
        #[bilrost(tag = 1006)]
        OneofUint64(u64),
        #[bilrost(tag = 1007)]
        OneofFloat(f32),
        #[bilrost(tag = 1008)]
        OneofDouble(f64),
        #[bilrost(tag = 1009)]
        OneofEnum(NestedEnum),
    }

    #[derive(Clone, PartialEq, Oneof)]
    pub enum OneofField {
        Empty,
        #[bilrost(tag = 2001)]
        OneofUint32(u32),
        #[bilrost(tag = 2002)]
        OneofNestedMessage(Box<NestedMessage>),
        #[bilrost(tag = 2003)]
        OneofString(String),
        #[bilrost(tag = 2004, encoder(vecblob))]
        OneofBytes(Vec<u8>),
        #[bilrost(tag = 2005)]
        OneofBool(bool),
        #[bilrost(tag = 2006)]
        OneofUint64(u64),
        #[bilrost(tag = 2007)]
        OneofFloat(f32),
        #[bilrost(tag = 2008)]
        OneofDouble(f64),
        #[bilrost(tag = 2009)]
        OneofEnum(NestedEnum),
    }
}
