#![cfg(test)]
extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{format, vec};
use bilrost::{DistinguishedMessage, Enumeration, Message, Oneof};
use core::default::Default;
use itertools::{repeat_n, Itertools};

#[test]
fn derived_message_field_ordering() {
    #[derive(Clone, Debug, PartialEq, Oneof)]
    enum A {
        #[bilrost(1)]
        One(bool),
        #[bilrost(10)]
        Ten(bool),
        #[bilrost(20)]
        Twenty(bool),
    }

    #[derive(Clone, Debug, PartialEq, Oneof)]
    enum B {
        #[bilrost(9)]
        Nine(bool),
        #[bilrost(11)]
        Eleven(bool),
    }

    #[derive(Clone, Debug, PartialEq, Oneof)]
    enum C {
        #[bilrost(13)]
        Thirteen(bool),
        #[bilrost(16)]
        Sixteen(bool),
        #[bilrost(22)]
        TwentyTwo(bool),
    }

    #[derive(Clone, Debug, PartialEq, Oneof)]
    enum D {
        #[bilrost(18)]
        Eighteen(bool),
        #[bilrost(19)]
        Nineteen(bool),
    }

    use A::*;
    use B::*;
    use C::*;
    use D::*;

    #[derive(Clone, Debug, PartialEq, Message)]
    struct Struct {
        #[bilrost(0)]
        zero: bool,
        #[bilrost(oneof = "1, 10, 20")]
        a: Option<A>,
        #[bilrost(4)]
        four: bool,
        #[bilrost(5)]
        five: bool,
        #[bilrost(oneof = "9, 11")]
        b: Option<B>,
        // implicitly tagged 12
        twelve: bool,
        #[bilrost(oneof = "13, 16, 22")]
        c: Option<C>,
        #[bilrost(14)]
        fourteen: bool,
        // implicitly tagged 15
        fifteen: bool,
        #[bilrost(17)]
        seventeen: bool,
        #[bilrost(oneof = "18, 19")]
        d: Option<D>,
        #[bilrost(21)]
        twentyone: bool,
        #[bilrost(50)]
        fifty: bool,
    }

    impl TryFrom<Vec<bool>> for Struct {
        type Error = ();

        fn try_from(value: Vec<bool>) -> Result<Self, ()> {
            match value[..] {
                [zero, four, five, twelve, fourteen, fifteen, seventeen, twentyone, fifty] => {
                    Ok(Self {
                        zero,
                        four,
                        five,
                        twelve,
                        fourteen,
                        fifteen,
                        seventeen,
                        twentyone,
                        fifty,
                        ..Default::default()
                    })
                }
                _ => Err(()),
            }
        }
    }

    // This must be the same as the number of fields we're putting into the struct...
    let bools = repeat_n([false, true], 9).multi_cartesian_product();
    let abcd = [None, Some(One(true)), Some(Ten(true)), Some(Twenty(true))]
        .into_iter()
        .cartesian_product([None, Some(Nine(true)), Some(Eleven(true))])
        .cartesian_product([
            None,
            Some(Thirteen(true)),
            Some(Sixteen(true)),
            Some(TwentyTwo(true)),
        ])
        .cartesian_product([None, Some(Eighteen(true)), Some(Nineteen(true))]);
    for (bools, oneofs) in bools.cartesian_product(abcd) {
        let mut out: Struct = bools.try_into().unwrap();
        (((out.a, out.b), out.c), out.d) = oneofs;

        let encoded_len = out.encoded_len();
        let encoded = out.encode_to_vec();
        assert_eq!(encoded.len(), encoded_len);
        let re = Struct::decode(&encoded[..]).unwrap();
        assert_eq!(out, re);
    }
}

#[test]
fn duplicated_field_decoding() {
    #[derive(Message)]
    struct Foo {
        #[bilrost(1)]
        a: Vec<bool>,
        #[bilrost(2)]
        b: Vec<bool>,
    }

    #[derive(Debug, PartialEq, Message)]
    struct Bar {
        #[bilrost(1)]
        a: Option<bool>,
        #[bilrost(2)]
        b: bool,
    }

    let a_single = Foo {
        a: vec![true],
        b: vec![],
    }
    .encode_to_vec();
    let a_duplicated = Foo {
        a: vec![true, false],
        b: vec![],
    }
    .encode_to_vec();
    let b_single = Foo {
        a: vec![],
        b: vec![true],
    }
    .encode_to_vec();
    let b_duplicated = Foo {
        a: vec![],
        b: vec![true, false],
    }
    .encode_to_vec();

    assert_eq!(
        Bar::decode(&a_single[..]),
        Ok(Bar {
            a: Some(true),
            b: false
        })
    );
    assert_eq!(
        Bar::decode(&a_duplicated[..]).unwrap_err().to_string(),
        "failed to decode Bilrost message: \
            Bar.a: multiple occurrences of non-repeated field"
    );
    assert_eq!(Bar::decode(&b_single[..]), Ok(Bar { a: None, b: true }));
    assert_eq!(
        Bar::decode(&b_duplicated[..]).unwrap_err().to_string(),
        "failed to decode Bilrost message: \
            Bar.b: multiple occurrences of non-repeated field"
    );
}

#[test]
fn duplicated_packed_decoding() {
    #[derive(Message)]
    struct Foo {
        #[bilrost(tag = 1, encoder = "unpacked<vecblob>")]
        a: Vec<Vec<u8>>,
    }

    #[derive(Debug, PartialEq, Message)]
    struct Bar {
        #[bilrost(tag = 1, encoder = "packed")]
        a: Vec<bool>,
    }

    let single = Foo { a: vec![vec![1]] }.encode_to_vec();
    let multiple = Foo {
        a: vec![vec![1, 0]],
    }
    .encode_to_vec();
    let duplicated = Foo {
        a: vec![vec![1], vec![0]],
    }
    .encode_to_vec();

    assert_eq!(Bar::decode(&single[..]), Ok(Bar { a: vec![true] }));
    assert_eq!(
        Bar::decode(&multiple[..]),
        Ok(Bar {
            a: vec![true, false]
        })
    );
    assert_eq!(
        Bar::decode(&duplicated[..]).unwrap_err().to_string(),
        "failed to decode Bilrost message: \
            Bar.a: multiple occurrences of packed repeated field"
    );
}

#[test]
fn oneof_field_decoding() {
    #[derive(Message)]
    struct Foo {
        #[bilrost(1)]
        a: Option<bool>,
        #[bilrost(2)]
        b: Option<bool>,
    }

    #[derive(Debug, PartialEq, Oneof)]
    enum AB {
        #[bilrost(1)]
        A(bool),
        #[bilrost(2)]
        B(bool),
    }
    use AB::*;

    #[derive(Debug, PartialEq, Message)]
    struct Bar {
        #[bilrost(oneof = "1, 2")]
        ab: Option<AB>,
    }

    let a_only = Foo {
        a: Some(true),
        b: None,
    }
    .encode_to_vec();
    let b_only = Foo {
        a: None,
        b: Some(false),
    }
    .encode_to_vec();
    let both = Foo {
        a: Some(false),
        b: Some(true),
    }
    .encode_to_vec();

    assert_eq!(Bar::decode(&a_only[..]), Ok(Bar { ab: Some(A(true)) }));
    assert_eq!(Bar::decode(&b_only[..]), Ok(Bar { ab: Some(B(false)) }));
    assert_eq!(
        Bar::decode(&both[..]).unwrap_err().to_string(),
        "failed to decode Bilrost message: Bar.ab: conflicting fields in oneof"
    );
}

#[test]
fn oneof_optioned_fields_encode_empty() {
    #[derive(Debug, PartialEq, Oneof)]
    enum Abc {
        #[bilrost(1)]
        A(String),
        #[bilrost(2)]
        B { named: u32 },
        #[bilrost(tag = 3, encoder = "packed")]
        C(Vec<bool>),
    }
    use Abc::*;

    #[derive(Debug, PartialEq, Message)]
    struct Foo {
        #[bilrost(oneof(1, 2, 3))]
        abc: Option<Abc>,
    }

    for value in [
        Foo { abc: None },
        Foo {
            abc: Some(A(Default::default())),
        },
        Foo {
            abc: Some(A("something".to_string())),
        },
        Foo {
            abc: Some(B {
                named: Default::default(),
            }),
        },
        Foo {
            abc: Some(B { named: 123 }),
        },
        Foo {
            abc: Some(C(Default::default())),
        },
        Foo {
            abc: Some(C(vec![false])),
        },
    ] {
        let encoded = value.encode_to_vec();
        let decoded = Foo::decode(encoded.as_slice()).unwrap();
        assert_eq!(value, decoded);
    }
}

#[test]
fn oneof_plain_fields_encode_empty() {
    /// Oneofs that have an empty variant
    #[derive(Debug, PartialEq, Oneof)]
    enum Abc {
        /// No fields
        Empty,
        #[bilrost(1)]
        A(String),
        #[bilrost(2)]
        B { named: u32 },
        #[bilrost(tag = 3, encoder = "packed")]
        C(Vec<bool>),
    }
    use Abc::*;

    #[derive(Debug, PartialEq, Message)]
    struct Foo {
        #[bilrost(oneof(1, 2, 3))]
        abc: Abc,
    }

    for value in [
        Foo { abc: Empty },
        Foo {
            abc: A(Default::default()),
        },
        Foo {
            abc: A("something".to_string()),
        },
        Foo {
            abc: B {
                named: Default::default(),
            },
        },
        Foo {
            abc: B { named: 123 },
        },
        Foo {
            abc: C(Default::default()),
        },
        Foo {
            abc: C(vec![false]),
        },
    ] {
        let encoded = value.encode_to_vec();
        let decoded = Foo::decode(encoded.as_slice()).unwrap();
        assert_eq!(value, decoded);
    }
}

#[test]
fn enumeration_decoding() {
    #[derive(Clone, Debug, PartialEq, Eq, Enumeration)]
    enum E {
        Five = 5,
        Ten = 10,
        Fifteen = 15,
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq, Enumeration)]
    enum F {
        #[default]
        Big = 1000,
        Bigger = 1_000_000,
        Biggest = 1_000_000_000,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Message, DistinguishedMessage)]
    struct EnumStruct {
        no_default: Option<E>,
        has_default: F,
    }

    for (no_default, has_default) in [None, Some(E::Five), Some(E::Ten), Some(E::Fifteen)]
        .into_iter()
        .cartesian_product([F::Big, F::Bigger, F::Biggest])
    {
        let out = EnumStruct {
            no_default,
            has_default,
        };
        let encoded_len = out.encoded_len();
        let encoded = out.encode_to_vec();
        assert_eq!(encoded.len(), encoded_len);
        let re = EnumStruct::decode(encoded.as_slice()).unwrap();
        assert_eq!(out, re);
        let re_distinguished = EnumStruct::decode_distinguished(encoded.as_slice()).unwrap();
        assert_eq!(out, re_distinguished);
    }
}

#[test]
fn enumeration_helpers() {
    #[derive(Clone, Debug, PartialEq, Eq, Enumeration)]
    enum E {
        Five = 5,
        Ten = 10,
        Fifteen = 15,
    }

    #[derive(Clone, Debug, PartialEq, Message)]
    struct HelpedStruct {
        #[bilrost(enumeration(E))]
        regular: u32,
        #[bilrost(enumeration(E))]
        optional: Option<u32>,
    }

    #[derive(Clone, Debug, PartialEq, Message)]
    struct StrictStruct {
        regular: Option<E>,
        optional: Option<E>,
    }

    let val = HelpedStruct {
        regular: 5,
        optional: Some(5),
    };
    assert_eq!(val.regular(), Ok(E::Five));
    assert_eq!(val.optional(), Some(Ok(E::Five)));

    let val = HelpedStruct {
        regular: 222,
        optional: Some(222),
    };
    assert_eq!(
        val.regular()
            .expect_err("bad enumeration value parsed successfully")
            .to_string(),
        "failed to decode Bilrost message: unknown enumeration value"
    );
    assert_eq!(
        val.optional()
            .unwrap()
            .expect_err("bad enumeration value parsed successfully")
            .to_string(),
        "failed to decode Bilrost message: unknown enumeration value"
    );

    let val = HelpedStruct::default();
    assert_eq!(val.optional(), None);

    // Demonstrate that the same errors happen when we decode to a struct with strict
    // enumeration fields, it just happens sooner.
    for (val, error_path) in [
        (
            HelpedStruct {
                regular: 222,
                optional: None,
            },
            "StrictStruct.regular",
        ),
        (
            HelpedStruct {
                regular: 5,
                optional: Some(222),
            },
            "StrictStruct.optional",
        ),
    ] {
        let encoded = val.encode_to_vec();
        let decoded = StrictStruct::decode(encoded.as_slice());
        assert_eq!(
            decoded
                .expect_err("decoded an invalid enumeration value without error")
                .to_string(),
            format!("failed to decode Bilrost message: {error_path}: unknown enumeration value")
        );
    }
}

#[test]
fn directly_included_message() {
    #[derive(Clone, Debug, PartialEq, Eq, Message, DistinguishedMessage)]
    struct Inner {
        a: String,
        b: i64,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Message, DistinguishedMessage)]
    struct OuterDirect {
        inner: Inner,
        also: String,
    }

    #[derive(Debug, PartialEq, Eq, Message, DistinguishedMessage)]
    struct OuterOptional {
        inner: Option<Inner>,
        also: Option<String>,
    }

    // With a directly included inner message field, it should encode only when its value is
    // defaulted.

    // When the inner message is default, it doesn't encode.
    let val = OuterDirect {
        inner: Default::default(),
        also: "abc".into(),
    };
    let encoded = val.encode_to_vec();
    assert_eq!(OuterDirect::decode(encoded.as_slice()), Ok(val.clone()));
    assert_eq!(
        OuterDirect::decode_distinguished(encoded.as_slice()),
        Ok(val.clone())
    );
    assert_eq!(
        OuterOptional::decode(encoded.as_slice()),
        Ok(OuterOptional {
            inner: None,
            also: Some("abc".into())
        })
    );
    assert_eq!(
        OuterOptional::decode_distinguished(encoded.as_slice()),
        Ok(OuterOptional {
            inner: None,
            also: Some("abc".into())
        })
    );

    // The inner message is included when it is not fully defaulted
    let val = OuterDirect {
        inner: Inner {
            a: "def".into(),
            b: 0,
        },
        also: "abc".into(),
    };
    let encoded = val.encode_to_vec();
    assert_eq!(OuterDirect::decode(encoded.as_slice()), Ok(val.clone()),);
    assert_eq!(
        OuterDirect::decode_distinguished(encoded.as_slice()),
        Ok(val.clone())
    );
    assert_eq!(
        OuterOptional::decode(encoded.as_slice()),
        Ok(OuterOptional {
            inner: Some(Inner {
                a: "def".into(),
                b: 0
            }),
            also: Some("abc".into())
        })
    );
    assert_eq!(
        OuterOptional::decode_distinguished(encoded.as_slice()),
        Ok(OuterOptional {
            inner: Some(Inner {
                a: "def".into(),
                b: 0
            }),
            also: Some("abc".into())
        })
    );
}
