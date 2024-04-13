use bytes::{Buf, BufMut};

use crate::buf::ReverseBuf;
use crate::encoding::{
    delegate_value_encoding, encode_varint, encoded_len_varint, encoder_where_value_encoder,
    prepend_varint, skip_field, Canonicity, Capped, DecodeContext, DistinguishedEncoder,
    DistinguishedValueEncoder, EmptyState, Encoder, General, TagReader, TagRevWriter, TagWriter,
    TrivialTagMeasurer, ValueEncoder, WireType, Wiretyped,
};
use crate::DecodeError;

/// Every other tuple type (up to arity 12) implements ValueEncoder for the encoding (E, ...) where
/// its elements are encoded by the corresponding sub-encoder. The representation on the wire is
/// exactly the same as if it were a message type that has fields with tags 0 through arity-1.
macro_rules! impl_tuple {
    (
        $arity:tt,
        $name:tt,
        $test_mod_name:ident,
        ($($numbers:tt),*),
        ($($numbers_desc:tt),*),
        ($($letters:ident),*),
        ($($letters_desc:ident),*),
        ($($encodings:ident),*),
    ) => {
        // All tuple types encode as nested messages, so all of them implement ValueEncoder and
        // should therefore implement Encoder in terms of that.
        encoder_where_value_encoder!(
            ($($encodings,)*),
            with generics ($($encodings),*)
        );

        impl<$($letters,)*> EmptyState for ($($letters,)*)
        where
            $($letters: EmptyState,)*
        {
            #[inline]
            fn empty() -> Self {
                ($($letters::empty(),)*)
            }

            #[inline]
            fn is_empty(&self) -> bool {
                true $(&& self.$numbers.is_empty())*
            }

            #[inline]
            fn clear(&mut self) {
                $(self.$numbers.clear();)*
            }
        }

        impl<$($letters,)* $($encodings,)*> Wiretyped<($($encodings,)*)> for ($($letters,)*) {
            const WIRE_TYPE: WireType = WireType::LengthDelimited;
        }

        impl<$($letters,)* $($encodings,)*> ValueEncoder<($($encodings,)*)> for ($($letters,)*)
        where
            $($letters: EmptyState + Encoder<$encodings>,)*
        {
            #[inline]
            fn encode_value<__B: BufMut + ?Sized>(value: &Self, buf: &mut __B) {
                // Because we do not implement tuples with more than arity 32, we can always use
                // the trivial tag measurer implementation.
                let tm = &mut TrivialTagMeasurer::new();
                let message_len = 0usize $(+ $letters::encoded_len($numbers, &value.$numbers, tm))*;
                encode_varint(message_len as u64, buf);
                let tw = &mut TagWriter::new();
                $($letters::encode($numbers, &value.$numbers, buf, tw);)*
            }

            #[inline]
            fn prepend_value<__B: ReverseBuf + ?Sized>(
                value: &Self,
                buf: &mut __B,
            ) {
                let end = buf.remaining();
                let tw = &mut TagRevWriter::new();
                $($letters_desc::prepend_encode($numbers_desc, &value.$numbers_desc, buf, tw);)*
                tw.finalize(buf);
                prepend_varint((buf.remaining() - end) as u64, buf);
            }

            #[inline]
            fn value_encoded_len(value: &Self) -> usize {
                // Because we do not implement tuples with more than arity 32, we can always use
                // the trivial tag measurer implementation.
                let tm = &mut TrivialTagMeasurer::new();
                let message_len = 0usize $(+ $letters::encoded_len($numbers, &value.$numbers, tm))*;
                encoded_len_varint(message_len as u64) + message_len
            }

            #[inline]
            fn decode_value<__B: Buf + ?Sized>(
                value: &mut Self,
                mut buf: Capped<__B>,
                ctx: DecodeContext,
            ) -> Result<(), DecodeError> {
                let mut buf = buf.take_length_delimited()?;
                ctx.limit_reached()?;
                let ctx = ctx.enter_recursion();
                let tr = &mut TagReader::new();
                let mut last_tag = None::<u32>;
                while buf.has_remaining()? {
                    let (tag, wire_type) = tr.decode_key(buf.lend())?;
                    let duplicated = last_tag == Some(tag);
                    last_tag = Some(tag);
                    // Decode the field. Each tuple field has a tag corresponding to its index.
                    match tag {
                        $($numbers => {
                            $letters::decode(
                                wire_type,
                                duplicated,
                                &mut value.$numbers,
                                buf.lend(),
                                ctx.clone(),
                            ).map_err(|mut error| {
                                error.push($name, stringify!($numbers));
                                error
                            })?
                        })*
                        _ => skip_field(wire_type, buf.lend())?,
                    }
                }
                Ok(())
            }
        }

        impl<$($letters,)* $($encodings,)*> DistinguishedValueEncoder<($($encodings,)*)>
        for ($($letters,)*)
        where
            Self: Eq,
            $($letters: Eq + EmptyState + DistinguishedEncoder<$encodings>,)*
        {
            #[inline]
            fn decode_value_distinguished<const ALLOW_EMPTY: bool>(
                value: &mut Self,
                mut buf: Capped<impl Buf + ?Sized>,
                ctx: DecodeContext,
            ) -> Result<Canonicity, DecodeError>
            where
                Self: Sized,
            {
                let mut buf = buf.take_length_delimited()?;
                // Since tuples emulate messages, empty values always encode and decode from zero
                // bytes. It is far cheaper to check here than to check after the value has been
                // decoded and checking the value's `is_empty()`.
                if !ALLOW_EMPTY && buf.remaining_before_cap() == 0 {
                    return Ok(Canonicity::NotCanonical);
                }
                ctx.limit_reached()?;
                let mut canon = Canonicity::Canonical;
                let ctx = ctx.enter_recursion();
                let tr = &mut TagReader::new();
                let mut last_tag = None::<u32>;
                while buf.has_remaining()? {
                    let (tag, wire_type) = tr.decode_key(buf.lend())?;
                    let duplicated = last_tag == Some(tag);
                    last_tag = Some(tag);
                    // Decode the field. Each tuple field has a tag corresponding to its index.
                    match tag {
                        $($numbers => {
                            canon.update($letters::decode_distinguished(
                                wire_type,
                                duplicated,
                                &mut value.$numbers,
                                buf.lend(),
                                ctx.clone(),
                            ).map_err(|mut error| {
                                error.push($name, stringify!($numbers));
                                error
                            })?);
                        })*
                        _ => {
                            skip_field(wire_type, buf.lend())?;
                            canon.update(Canonicity::HasExtensions);
                        },
                    }
                }
                Ok(canon)
            }
        }

        #[cfg(test)]
        mod $test_mod_name {
            mod delegated_bools {
                use crate::encoding::General;
                use crate::encoding::test::check_type_test;
                $(type $letters = bool;)*

                check_type_test!(
                    General,
                    expedient,
                    from [bool; $arity],
                    into ($($letters,)*),
                    WireType::LengthDelimited
                );
                check_type_test!(
                    General,
                    distinguished,
                    from [bool; $arity],
                    into ($($letters,)*),
                    WireType::LengthDelimited
                );
            }
            mod varint_bools {
                use crate::encoding::test::check_type_test;
                $(type $letters = bool;)*
                $(type $encodings = crate::encoding::Varint;)*

                check_type_test!(
                    ($($encodings,)*),
                    expedient,
                    from [bool; $arity],
                    into ($($letters,)*),
                    WireType::LengthDelimited
                );
                check_type_test!(
                    ($($encodings,)*),
                    distinguished,
                    from [bool; $arity],
                    into ($($letters,)*),
                    WireType::LengthDelimited
                );
            }
            mod fixed_floats {
                use crate::encoding::test::check_type_test;
                $(type $letters = f32;)*
                $(type $encodings = crate::encoding::Fixed;)*

                check_type_test!(
                    ($($encodings,)*),
                    expedient,
                    from [f32; $arity],
                    into ($($letters,)*),
                    WireType::LengthDelimited
                );
            }
            mod small_arrays {
                use crate::encoding::test::check_type_test;
                $(type $letters = [u8; 1];)*
                $(type $encodings = crate::encoding::PlainBytes;)*

                check_type_test!(
                    ($($encodings,)*),
                    expedient,
                    from [[u8; 1]; $arity],
                    into ($($letters,)*),
                    WireType::LengthDelimited
                );
                check_type_test!(
                    ($($encodings,)*),
                    distinguished,
                    from [[u8; 1]; $arity],
                    into ($($letters,)*),
                    WireType::LengthDelimited
                );
            }
        }
    }
}

impl_tuple!(
    1,             //
    "(1-tuple)",   //
    tuple_arity_1, //
    (0),           //
    (0),           //
    (A),           //
    (A),           //
    (Ae),          //
);
impl_tuple!(
    2,             //
    "(2-tuple)",   //
    tuple_arity_2, //
    (0, 1),        //
    (1, 0),        //
    (A, B),        //
    (B, A),        //
    (Ae, Be),      //
);
impl_tuple!(
    3,             //
    "(3-tuple)",   //
    tuple_arity_3, //
    (0, 1, 2),     //
    (2, 1, 0),     //
    (A, B, C),     //
    (C, B, A),     //
    (Ae, Be, Ce),  //
);
impl_tuple!(
    4,                //
    "(4-tuple)",      //
    tuple_arity_4,    //
    (0, 1, 2, 3),     //
    (3, 2, 1, 0),     //
    (A, B, C, D),     //
    (D, C, B, A),     //
    (Ae, Be, Ce, De), //
);
impl_tuple!(
    5,                    //
    "(5-tuple)",          //
    tuple_arity_5,        //
    (0, 1, 2, 3, 4),      //
    (4, 3, 2, 1, 0),      //
    (A, B, C, D, E),      //
    (E, D, C, B, A),      //
    (Ae, Be, Ce, De, Ee), //
);
impl_tuple!(
    6,                        //
    "(6-tuple)",              //
    tuple_arity_6,            //
    (0, 1, 2, 3, 4, 5),       //
    (5, 4, 3, 2, 1, 0),       //
    (A, B, C, D, E, F),       //
    (F, E, D, C, B, A),       //
    (Ae, Be, Ce, De, Ee, Fe), //
);
impl_tuple!(
    7,                            //
    "(7-tuple)",                  //
    tuple_arity_7,                //
    (0, 1, 2, 3, 4, 5, 6),        //
    (6, 5, 4, 3, 2, 1, 0),        //
    (A, B, C, D, E, F, G),        //
    (G, F, E, D, C, B, A),        //
    (Ae, Be, Ce, De, Ee, Fe, Ge), //
);
impl_tuple!(
    8,                                //
    "(8-tuple)",                      //
    tuple_arity_8,                    //
    (0, 1, 2, 3, 4, 5, 6, 7),         //
    (7, 6, 5, 4, 3, 2, 1, 0),         //
    (A, B, C, D, E, F, G, H),         //
    (H, G, F, E, D, C, B, A),         //
    (Ae, Be, Ce, De, Ee, Fe, Ge, He), //
);
impl_tuple!(
    9,                                    //
    "(9-tuple)",                          //
    tuple_arity_9,                        //
    (0, 1, 2, 3, 4, 5, 6, 7, 8),          //
    (8, 7, 6, 5, 4, 3, 2, 1, 0),          //
    (A, B, C, D, E, F, G, H, I),          //
    (I, H, G, F, E, D, C, B, A),          //
    (Ae, Be, Ce, De, Ee, Fe, Ge, He, Ie), //
);
impl_tuple!(
    10,                                       //
    "(10-tuple)",                             //
    tuple_arity_10,                           //
    (0, 1, 2, 3, 4, 5, 6, 7, 8, 9),           //
    (9, 8, 7, 6, 5, 4, 3, 2, 1, 0),           //
    (A, B, C, D, E, F, G, H, I, J),           //
    (J, I, H, G, F, E, D, C, B, A),           //
    (Ae, Be, Ce, De, Ee, Fe, Ge, He, Ie, Je), //
);
impl_tuple!(
    11,                                           //
    "(11-tuple)",                                 //
    tuple_arity_11,                               //
    (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10),           //
    (10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0),           //
    (A, B, C, D, E, F, G, H, I, J, K),            //
    (K, J, I, H, G, F, E, D, C, B, A),            //
    (Ae, Be, Ce, De, Ee, Fe, Ge, He, Ie, Je, Ke), //
);
impl_tuple!(
    12,                                               //
    "(12-tuple)",                                     //
    tuple_arity_12,                                   //
    (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11),           //
    (11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0),           //
    (A, B, C, D, E, F, G, H, I, J, K, L),             //
    (L, K, J, I, H, G, F, E, D, C, B, A),             //
    (Ae, Be, Ce, De, Ee, Fe, Ge, He, Ie, Je, Ke, Le), //
);

delegate_value_encoding!(
    delegate from (General) to ((General,))
    for type ((A,)) including distinguished
    with generics (A)
);
delegate_value_encoding!(
    delegate from (General) to ((General, General))
    for type ((A, B)) including distinguished
    with generics (A, B)
);
delegate_value_encoding!(
    delegate from (General) to ((General, General, General))
    for type ((A, B, C)) including distinguished
    with generics (A, B, C)
);
delegate_value_encoding!(
    delegate from (General) to ((General, General, General, General))
    for type ((A, B, C, D)) including distinguished
    with generics (A, B, C, D)
);
delegate_value_encoding!(
    delegate from (General) to ((General, General, General, General, General))
    for type ((A, B, C, D, E)) including distinguished
    with generics (A, B, C, D, E)
);
delegate_value_encoding!(
    delegate from (General) to ((General, General, General, General, General, General))
    for type ((A, B, C, D, E, F)) including distinguished
    with generics (A, B, C, D, E, F)
);
delegate_value_encoding!(
    delegate from (General) to ((General, General, General, General, General, General,
                                 General))
    for type ((A, B, C, D, E, F, G)) including distinguished
    with generics (A, B, C, D, E, F, G)
);
delegate_value_encoding!(
    delegate from (General) to ((General, General, General, General, General, General,
                                 General, General))
    for type ((A, B, C, D, E, F, G, H)) including distinguished
    with generics (A, B, C, D, E, F, G, H)
);
delegate_value_encoding!(
    delegate from (General) to ((General, General, General, General, General, General,
                                 General, General, General))
    for type ((A, B, C, D, E, F, G, H, I)) including distinguished
    with generics (A, B, C, D, E, F, G, H, I)
);
delegate_value_encoding!(
    delegate from (General) to ((General, General, General, General, General, General,
                                 General, General, General, General))
    for type ((A, B, C, D, E, F, G, H, I, J)) including distinguished
    with generics (A, B, C, D, E, F, G, H, I, J)
);
delegate_value_encoding!(
    delegate from (General) to ((General, General, General, General, General, General,
                                 General, General, General, General, General))
    for type ((A, B, C, D, E, F, G, H, I, J, K)) including distinguished
    with generics (A, B, C, D, E, F, G, H, I, J, K)
);
delegate_value_encoding!(
    delegate from (General) to ((General, General, General, General, General, General,
                                 General, General, General, General, General, General))
    for type ((A, B, C, D, E, F, G, H, I, J, K, L)) including distinguished
    with generics (A, B, C, D, E, F, G, H, I, J, K, L)
);
