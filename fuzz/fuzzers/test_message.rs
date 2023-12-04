/// This proto includes every type of field in both singular and repeated
/// forms.
///
/// Also, crucially, all messages and enums in this file are eventually
/// submessages of this message.  So for example, a fuzz test of TestAllTypes
/// could trigger bugs that occur in any message type in this file.  We verify
/// this stays true in a unit test.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::bilrost::Message)]
pub struct TestAllTypesProto3 {
    /// Singular
    #[bilrost(sint32, tag = "1")]
    pub optional_sint32: i32,
    #[bilrost(sint64, tag = "2")]
    pub optional_sint64: i64,
    #[bilrost(uint32, tag = "3")]
    pub optional_uint32: u32,
    #[bilrost(uint64, tag = "4")]
    pub optional_uint64: u64,
    #[bilrost(ufixed32, tag = "7")]
    pub optional_ufixed32: u32,
    #[bilrost(ufixed64, tag = "8")]
    pub optional_ufixed64: u64,
    #[bilrost(sfixed32, tag = "9")]
    pub optional_sfixed32: i32,
    #[bilrost(sfixed64, tag = "10")]
    pub optional_sfixed64: i64,
    #[bilrost(float32, tag = "11")]
    pub optional_float32: f32,
    #[bilrost(float64, tag = "12")]
    pub optional_float64: f64,
    #[bilrost(bool, tag = "13")]
    pub optional_bool: bool,
    #[bilrost(string, tag = "14")]
    pub optional_string: ::bilrost::alloc::string::String,
    #[bilrost(bytes = "vec", tag = "15")]
    pub optional_bytes: ::bilrost::alloc::vec::Vec<u8>,
    #[bilrost(message, optional, boxed, tag = "18")]
    pub optional_nested_message: ::core::option::Option<
        ::bilrost::alloc::boxed::Box<test_message::NestedMessage>,
    >,
    #[bilrost(message, optional, tag = "19")]
    pub optional_foreign_message: ::core::option::Option<ForeignMessage>,
    #[bilrost(enumeration = "test_message::NestedEnum", tag = "21")]
    pub optional_nested_enum: u32,
    #[bilrost(enumeration = "ForeignEnum", tag = "22")]
    pub optional_foreign_enum: u32,
    #[bilrost(enumeration = "test_message::AliasedEnum", tag = "23")]
    pub optional_aliased_enum: u32,
    #[bilrost(string, tag = "24")]
    pub optional_string_piece: ::bilrost::alloc::string::String,
    #[bilrost(string, tag = "25")]
    pub optional_cord: ::bilrost::alloc::string::String,
    #[bilrost(message, optional, boxed, tag = "27")]
    pub recursive_message: ::core::option::Option<
        ::bilrost::alloc::boxed::Box<TestAllTypesProto3>,
    >,
    /// Repeated
    #[bilrost(sint32, repeated, tag = "31")]
    pub repeated_sint32: ::bilrost::alloc::vec::Vec<i32>,
    #[bilrost(sint64, repeated, tag = "32")]
    pub repeated_sint64: ::bilrost::alloc::vec::Vec<i64>,
    #[bilrost(uint32, repeated, tag = "33")]
    pub repeated_uint32: ::bilrost::alloc::vec::Vec<u32>,
    #[bilrost(uint64, repeated, tag = "34")]
    pub repeated_uint64: ::bilrost::alloc::vec::Vec<u64>,
    #[bilrost(ufixed32, repeated, tag = "37")]
    pub repeated_ufixed32: ::bilrost::alloc::vec::Vec<u32>,
    #[bilrost(ufixed64, repeated, tag = "38")]
    pub repeated_ufixed64: ::bilrost::alloc::vec::Vec<u64>,
    #[bilrost(sfixed32, repeated, tag = "39")]
    pub repeated_sfixed32: ::bilrost::alloc::vec::Vec<i32>,
    #[bilrost(sfixed64, repeated, tag = "40")]
    pub repeated_sfixed64: ::bilrost::alloc::vec::Vec<i64>,
    #[bilrost(float32, repeated, tag = "41")]
    pub repeated_float32: ::bilrost::alloc::vec::Vec<f32>,
    #[bilrost(float64, repeated, tag = "42")]
    pub repeated_float64: ::bilrost::alloc::vec::Vec<f64>,
    #[bilrost(bool, repeated, tag = "43")]
    pub repeated_bool: ::bilrost::alloc::vec::Vec<bool>,
    #[bilrost(string, repeated, tag = "44")]
    pub repeated_string: ::bilrost::alloc::vec::Vec<::bilrost::alloc::string::String>,
    #[bilrost(bytes = "vec", repeated, tag = "45")]
    pub repeated_bytes: ::bilrost::alloc::vec::Vec<::bilrost::alloc::vec::Vec<u8>>,
    #[bilrost(message, repeated, tag = "48")]
    pub repeated_nested_message: ::bilrost::alloc::vec::Vec<
        test_message::NestedMessage,
    >,
    #[bilrost(message, repeated, tag = "49")]
    pub repeated_foreign_message: ::bilrost::alloc::vec::Vec<ForeignMessage>,
    #[bilrost(enumeration = "test_message::NestedEnum", repeated, tag = "51")]
    pub repeated_nested_enum: ::bilrost::alloc::vec::Vec<u32>,
    #[bilrost(enumeration = "ForeignEnum", repeated, tag = "52")]
    pub repeated_foreign_enum: ::bilrost::alloc::vec::Vec<u32>,
    #[bilrost(string, repeated, tag = "54")]
    pub repeated_string_piece: ::bilrost::alloc::vec::Vec<::bilrost::alloc::string::String>,
    #[bilrost(string, repeated, tag = "55")]
    pub repeated_cord: ::bilrost::alloc::vec::Vec<::bilrost::alloc::string::String>,
    /// Packed
    #[bilrost(sint32, repeated, tag = "75")]
    pub packed_sint32: ::bilrost::alloc::vec::Vec<i32>,
    #[bilrost(sint64, repeated, tag = "76")]
    pub packed_sint64: ::bilrost::alloc::vec::Vec<i64>,
    #[bilrost(uint32, repeated, tag = "77")]
    pub packed_uint32: ::bilrost::alloc::vec::Vec<u32>,
    #[bilrost(uint64, repeated, tag = "78")]
    pub packed_uint64: ::bilrost::alloc::vec::Vec<u64>,
    #[bilrost(ufixed32, repeated, tag = "81")]
    pub packed_ufixed32: ::bilrost::alloc::vec::Vec<u32>,
    #[bilrost(ufixed64, repeated, tag = "82")]
    pub packed_ufixed64: ::bilrost::alloc::vec::Vec<u64>,
    #[bilrost(sfixed32, repeated, tag = "83")]
    pub packed_sfixed32: ::bilrost::alloc::vec::Vec<i32>,
    #[bilrost(sfixed64, repeated, tag = "84")]
    pub packed_sfixed64: ::bilrost::alloc::vec::Vec<i64>,
    #[bilrost(float32, repeated, tag = "85")]
    pub packed_float32: ::bilrost::alloc::vec::Vec<f32>,
    #[bilrost(float64, repeated, tag = "86")]
    pub packed_float64: ::bilrost::alloc::vec::Vec<f64>,
    #[bilrost(bool, repeated, tag = "87")]
    pub packed_bool: ::bilrost::alloc::vec::Vec<bool>,
    #[bilrost(enumeration = "test_message::NestedEnum", repeated, tag = "88")]
    pub packed_nested_enum: ::bilrost::alloc::vec::Vec<u32>,
    /// Unpacked
    #[bilrost(sint32, repeated, packed = "false", tag = "89")]
    pub unpacked_sint32: ::bilrost::alloc::vec::Vec<i32>,
    #[bilrost(sint64, repeated, packed = "false", tag = "90")]
    pub unpacked_sint64: ::bilrost::alloc::vec::Vec<i64>,
    #[bilrost(uint32, repeated, packed = "false", tag = "91")]
    pub unpacked_uint32: ::bilrost::alloc::vec::Vec<u32>,
    #[bilrost(uint64, repeated, packed = "false", tag = "92")]
    pub unpacked_uint64: ::bilrost::alloc::vec::Vec<u64>,
    #[bilrost(ufixed32, repeated, packed = "false", tag = "95")]
    pub unpacked_ufixed32: ::bilrost::alloc::vec::Vec<u32>,
    #[bilrost(ufixed64, repeated, packed = "false", tag = "96")]
    pub unpacked_ufixed64: ::bilrost::alloc::vec::Vec<u64>,
    #[bilrost(sfixed32, repeated, packed = "false", tag = "97")]
    pub unpacked_sfixed32: ::bilrost::alloc::vec::Vec<i32>,
    #[bilrost(sfixed64, repeated, packed = "false", tag = "98")]
    pub unpacked_sfixed64: ::bilrost::alloc::vec::Vec<i64>,
    #[bilrost(float32, repeated, packed = "false", tag = "99")]
    pub unpacked_float32: ::bilrost::alloc::vec::Vec<f32>,
    #[bilrost(float64, repeated, packed = "false", tag = "100")]
    pub unpacked_float64: ::bilrost::alloc::vec::Vec<f64>,
    #[bilrost(bool, repeated, packed = "false", tag = "101")]
    pub unpacked_bool: ::bilrost::alloc::vec::Vec<bool>,
    #[bilrost(
        enumeration = "test_message::NestedEnum",
        repeated,
        packed = "false",
        tag = "102"
    )]
    pub unpacked_nested_enum: ::bilrost::alloc::vec::Vec<u32>,
    /// Map
    #[bilrost(btree_map = "sint32, sint32", tag = "56")]
    pub map_sint32_sint32: ::bilrost::alloc::collections::BTreeMap<i32, i32>,
    #[bilrost(btree_map = "sint64, sint64", tag = "57")]
    pub map_sint64_sint64: ::bilrost::alloc::collections::BTreeMap<i64, i64>,
    #[bilrost(btree_map = "uint32, uint32", tag = "58")]
    pub map_uint32_uint32: ::bilrost::alloc::collections::BTreeMap<u32, u32>,
    #[bilrost(btree_map = "uint64, uint64", tag = "59")]
    pub map_uint64_uint64: ::bilrost::alloc::collections::BTreeMap<u64, u64>,
    #[bilrost(btree_map = "ufixed32, ufixed32", tag = "62")]
    pub map_ufixed32_ufixed32: ::bilrost::alloc::collections::BTreeMap<u32, u32>,
    #[bilrost(btree_map = "ufixed64, ufixed64", tag = "63")]
    pub map_ufixed64_ufixed64: ::bilrost::alloc::collections::BTreeMap<u64, u64>,
    #[bilrost(btree_map = "sfixed32, sfixed32", tag = "64")]
    pub map_sfixed32_sfixed32: ::bilrost::alloc::collections::BTreeMap<i32, i32>,
    #[bilrost(btree_map = "sfixed64, sfixed64", tag = "65")]
    pub map_sfixed64_sfixed64: ::bilrost::alloc::collections::BTreeMap<i64, i64>,
    #[bilrost(btree_map = "sint32, float32", tag = "66")]
    pub map_sint32_float32: ::bilrost::alloc::collections::BTreeMap<i32, f32>,
    #[bilrost(btree_map = "sint32, float64", tag = "67")]
    pub map_sint32_float64: ::bilrost::alloc::collections::BTreeMap<i32, f64>,
    #[bilrost(btree_map = "bool, bool", tag = "68")]
    pub map_bool_bool: ::bilrost::alloc::collections::BTreeMap<bool, bool>,
    #[bilrost(btree_map = "string, string", tag = "69")]
    pub map_string_string: ::bilrost::alloc::collections::BTreeMap<
        ::bilrost::alloc::string::String,
        ::bilrost::alloc::string::String,
    >,
    #[bilrost(btree_map = "string, bytes", tag = "70")]
    pub map_string_bytes: ::bilrost::alloc::collections::BTreeMap<
        ::bilrost::alloc::string::String,
        ::bilrost::alloc::vec::Vec<u8>,
    >,
    #[bilrost(btree_map = "string, message", tag = "71")]
    pub map_string_nested_message: ::bilrost::alloc::collections::BTreeMap<
        ::bilrost::alloc::string::String,
        test_message::NestedMessage,
    >,
    #[bilrost(btree_map = "string, message", tag = "72")]
    pub map_string_foreign_message: ::bilrost::alloc::collections::BTreeMap<
        ::bilrost::alloc::string::String,
        ForeignMessage,
    >,
    #[bilrost(
        btree_map = "string, enumeration(test_message::NestedEnum)",
        tag = "73"
    )]
    pub map_string_nested_enum: ::bilrost::alloc::collections::BTreeMap<
        ::bilrost::alloc::string::String,
        u32,
    >,
    #[bilrost(btree_map = "string, enumeration(ForeignEnum)", tag = "74")]
    pub map_string_foreign_enum: ::bilrost::alloc::collections::BTreeMap<
        ::bilrost::alloc::string::String,
        u32,
    >,
    /// Well-known types
    #[bilrost(message, optional, tag = "201")]
    pub optional_bool_wrapper: ::core::option::Option<bool>,
    #[bilrost(message, optional, tag = "202")]
    pub optional_sint32_wrapper: ::core::option::Option<i32>,
    #[bilrost(message, optional, tag = "203")]
    pub optional_sint64_wrapper: ::core::option::Option<i64>,
    #[bilrost(message, optional, tag = "204")]
    pub optional_uint32_wrapper: ::core::option::Option<u32>,
    #[bilrost(message, optional, tag = "205")]
    pub optional_uint64_wrapper: ::core::option::Option<u64>,
    #[bilrost(message, optional, tag = "206")]
    pub optional_float32_wrapper: ::core::option::Option<f32>,
    #[bilrost(message, optional, tag = "207")]
    pub optional_float64_wrapper: ::core::option::Option<f64>,
    #[bilrost(message, optional, tag = "208")]
    pub optional_string_wrapper: ::core::option::Option<::bilrost::alloc::string::String>,
    #[bilrost(message, optional, tag = "209")]
    pub optional_bytes_wrapper: ::core::option::Option<::bilrost::alloc::vec::Vec<u8>>,
    #[bilrost(message, repeated, tag = "211")]
    pub repeated_bool_wrapper: ::bilrost::alloc::vec::Vec<bool>,
    #[bilrost(message, repeated, tag = "212")]
    pub repeated_sint32_wrapper: ::bilrost::alloc::vec::Vec<i32>,
    #[bilrost(message, repeated, tag = "213")]
    pub repeated_sint64_wrapper: ::bilrost::alloc::vec::Vec<i64>,
    #[bilrost(message, repeated, tag = "214")]
    pub repeated_uint32_wrapper: ::bilrost::alloc::vec::Vec<u32>,
    #[bilrost(message, repeated, tag = "215")]
    pub repeated_uint64_wrapper: ::bilrost::alloc::vec::Vec<u64>,
    #[bilrost(message, repeated, tag = "216")]
    pub repeated_float32_wrapper: ::bilrost::alloc::vec::Vec<f32>,
    #[bilrost(message, repeated, tag = "217")]
    pub repeated_float64_wrapper: ::bilrost::alloc::vec::Vec<f64>,
    #[bilrost(message, repeated, tag = "218")]
    pub repeated_string_wrapper: ::bilrost::alloc::vec::Vec<
        ::bilrost::alloc::string::String,
    >,
    #[bilrost(message, repeated, tag = "219")]
    pub repeated_bytes_wrapper: ::bilrost::alloc::vec::Vec<::bilrost::alloc::vec::Vec<u8>>,
    #[bilrost(message, optional, tag = "301")]
    pub optional_duration: ::core::option::Option<::bilrost_types::Duration>,
    #[bilrost(message, optional, tag = "302")]
    pub optional_timestamp: ::core::option::Option<::bilrost_types::Timestamp>,
    #[bilrost(message, optional, tag = "303")]
    pub optional_field_mask: ::core::option::Option<::bilrost_types::FieldMask>,
    #[bilrost(message, optional, tag = "304")]
    pub optional_struct: ::core::option::Option<::bilrost_types::Struct>,
    #[bilrost(message, optional, tag = "305")]
    pub optional_any: ::core::option::Option<::bilrost_types::Any>,
    #[bilrost(message, optional, tag = "306")]
    pub optional_value: ::core::option::Option<::bilrost_types::Value>,
    #[bilrost(enumeration = "::bilrost_types::NullValue", tag = "307")]
    pub optional_null_value: u32,
    #[bilrost(message, repeated, tag = "311")]
    pub repeated_duration: ::bilrost::alloc::vec::Vec<::bilrost_types::Duration>,
    #[bilrost(message, repeated, tag = "312")]
    pub repeated_timestamp: ::bilrost::alloc::vec::Vec<::bilrost_types::Timestamp>,
    #[bilrost(message, repeated, tag = "313")]
    pub repeated_fieldmask: ::bilrost::alloc::vec::Vec<::bilrost_types::FieldMask>,
    #[bilrost(message, repeated, tag = "324")]
    pub repeated_struct: ::bilrost::alloc::vec::Vec<::bilrost_types::Struct>,
    #[bilrost(message, repeated, tag = "315")]
    pub repeated_any: ::bilrost::alloc::vec::Vec<::bilrost_types::Any>,
    #[bilrost(message, repeated, tag = "316")]
    pub repeated_value: ::bilrost::alloc::vec::Vec<::bilrost_types::Value>,
    #[bilrost(message, repeated, tag = "317")]
    pub repeated_list_value: ::bilrost::alloc::vec::Vec<::bilrost_types::ListValue>,
    /// Test field-name-to-JSON-name convention.
    /// (protobuf says names can be any valid C/C++ identifier.)
    #[bilrost(sint32, tag = "401")]
    pub fieldname1: i32,
    #[bilrost(sint32, tag = "402")]
    pub field_name2: i32,
    #[bilrost(sint32, tag = "403")]
    pub field_name3: i32,
    #[bilrost(sint32, tag = "404")]
    pub field_name4: i32,
    #[bilrost(sint32, tag = "405")]
    pub field0name5: i32,
    #[bilrost(sint32, tag = "406")]
    pub field_0_name6: i32,
    #[bilrost(sint32, tag = "407")]
    pub field_name7: i32,
    #[bilrost(sint32, tag = "408")]
    pub field_name8: i32,
    #[bilrost(sint32, tag = "409")]
    pub field_name9: i32,
    #[bilrost(sint32, tag = "410")]
    pub field_name10: i32,
    #[bilrost(sint32, tag = "411")]
    pub field_name11: i32,
    #[bilrost(sint32, tag = "412")]
    pub field_name12: i32,
    #[bilrost(sint32, tag = "413")]
    pub field_name13: i32,
    #[bilrost(sint32, tag = "414")]
    pub field_name14: i32,
    #[bilrost(sint32, tag = "415")]
    pub field_name15: i32,
    #[bilrost(sint32, tag = "416")]
    pub field_name16: i32,
    #[bilrost(sint32, tag = "417")]
    pub field_name17: i32,
    #[bilrost(sint32, tag = "418")]
    pub field_name18: i32,
    #[bilrost(
        oneof = "test_message::OneofField",
        tags = "111, 112, 113, 114, 115, 116, 117, 118, 119, 120"
    )]
    pub oneof_field: ::core::option::Option<test_message::OneofField>,
}
/// Nested message and enum types in `TestAllTypesProto3`.
pub mod test_message {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::bilrost::Message)]
    pub struct NestedMessage {
        #[bilrost(sint32, tag = "1")]
        pub a: i32,
        #[bilrost(message, optional, boxed, tag = "2")]
        pub corecursive: ::core::option::Option<
            ::bilrost::alloc::boxed::Box<super::TestAllTypesProto3>,
        >,
    }
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::bilrost::Enumeration
    )]
    #[repr(u32)]
    pub enum NestedEnum {
        Foo = 0,
        Bar = 1,
        Baz = 2,
        Max = u32::MAX,
    }
    impl NestedEnum {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                NestedEnum::Foo => "FOO",
                NestedEnum::Bar => "BAR",
                NestedEnum::Baz => "BAZ",
                NestedEnum::Max => "MAX",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "FOO" => Some(Self::Foo),
                "BAR" => Some(Self::Bar),
                "BAZ" => Some(Self::Baz),
                "MAX" => Some(Self::Max),
                _ => None,
            }
        }
    }
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::bilrost::Enumeration
    )]
    #[repr(i32)]
    pub enum AliasedEnum {
        AliasFoo = 0,
        AliasBar = 1,
        AliasBaz = 2,
    }
    impl AliasedEnum {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                AliasedEnum::AliasFoo => "ALIAS_FOO",
                AliasedEnum::AliasBar => "ALIAS_BAR",
                AliasedEnum::AliasBaz => "ALIAS_BAZ",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "ALIAS_FOO" => Some(Self::AliasFoo),
                "ALIAS_BAR" => Some(Self::AliasBar),
                "ALIAS_BAZ" => Some(Self::AliasBaz),
                _ => None,
            }
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::bilrost::Oneof)]
    pub enum OneofField {
        #[bilrost(uint32, tag = "111")]
        OneofUint32(u32),
        #[bilrost(message, tag = "112")]
        OneofNestedMessage(::bilrost::alloc::boxed::Box<NestedMessage>),
        #[bilrost(string, tag = "113")]
        OneofString(::bilrost::alloc::string::String),
        #[bilrost(bytes, tag = "114")]
        OneofBytes(::bilrost::alloc::vec::Vec<u8>),
        #[bilrost(bool, tag = "115")]
        OneofBool(bool),
        #[bilrost(uint64, tag = "116")]
        OneofUint64(u64),
        #[bilrost(float32, tag = "117")]
        OneofFloat(f32),
        #[bilrost(float64, tag = "118")]
        OneofDouble(f64),
        #[bilrost(enumeration = "NestedEnum", tag = "119")]
        OneofEnum(i32),
        #[bilrost(enumeration = "::bilrost_types::NullValue", tag = "120")]
        OneofNullValue(i32),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::bilrost::Message)]
pub struct ForeignMessage {
    #[bilrost(sint32, tag = "1")]
    pub c: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::bilrost::Enumeration)]
#[repr(i32)]
pub enum ForeignEnum {
    ForeignFoo = 0,
    ForeignBar = 1,
    ForeignBaz = 2,
}
impl ForeignEnum {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ForeignEnum::ForeignFoo => "FOREIGN_FOO",
            ForeignEnum::ForeignBar => "FOREIGN_BAR",
            ForeignEnum::ForeignBaz => "FOREIGN_BAZ",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "FOREIGN_FOO" => Some(Self::ForeignFoo),
            "FOREIGN_BAR" => Some(Self::ForeignBar),
            "FOREIGN_BAZ" => Some(Self::ForeignBaz),
            _ => None,
        }
    }
}
