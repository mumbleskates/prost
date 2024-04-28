use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::ops::Deref;

use crate::buf::ReverseBuf;
use bytes::{Buf, BufMut};

use crate::encoding::{
    const_varint, delegate_encoding, encode_varint, encoded_len_varint,
    encoder_where_value_encoder, prepend_varint, Canonicity, Capped, DecodeContext, DecodeError,
    DistinguishedValueEncoder, EmptyState, Encoder, ValueEncoder, WireType, Wiretyped,
};
use crate::DecodeErrorKind::InvalidValue;

/// `PlainBytes` implements encoding for blob values directly into `Vec<u8>`, and provides the base
/// implementation for that functionality. `Vec<u8>` cannot generically dispatch to `General`'s
/// encoding, since `General` already generically implements encoding for other kinds of `Vec`, but
/// this encoder can be used instead if it's desirable to have a value whose type is exactly
/// `Vec<u8>`.
pub struct PlainBytes;

encoder_where_value_encoder!(PlainBytes);

impl Wiretyped<PlainBytes> for Vec<u8> {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}

impl ValueEncoder<PlainBytes> for Vec<u8> {
    #[inline]
    fn encode_value<B: BufMut + ?Sized>(value: &Vec<u8>, buf: &mut B) {
        encode_varint(value.len() as u64, buf);
        buf.put_slice(value.as_slice());
    }

    #[inline]
    fn prepend_value<B: ReverseBuf + ?Sized>(value: &Vec<u8>, buf: &mut B) {
        buf.prepend_slice(value);
        prepend_varint(value.len() as u64, buf);
    }

    #[inline]
    fn value_encoded_len(value: &Vec<u8>) -> usize {
        encoded_len_varint(value.len() as u64) + value.len()
    }

    fn decode_value<B: Buf + ?Sized>(
        value: &mut Vec<u8>,
        mut buf: Capped<B>,
        _ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        let buf = buf.take_length_delimited()?;
        value.clear();
        value.reserve(buf.remaining_before_cap());
        value.put(buf.take_all());
        Ok(())
    }
}

impl DistinguishedValueEncoder<PlainBytes> for Vec<u8> {
    fn decode_value_distinguished<const ALLOW_EMPTY: bool>(
        value: &mut Vec<u8>,
        buf: Capped<impl Buf + ?Sized>,
        ctx: DecodeContext,
    ) -> Result<Canonicity, DecodeError> {
        ValueEncoder::<PlainBytes>::decode_value(value, buf, ctx)?;
        Ok(if !ALLOW_EMPTY && value.is_empty() {
            Canonicity::NotCanonical
        } else {
            Canonicity::Canonical
        })
    }
}

delegate_encoding!(delegate from (PlainBytes) to (crate::encoding::Unpacked<PlainBytes>)
    for type (Vec<Vec<u8>>) including distinguished);
delegate_encoding!(delegate from (PlainBytes) to (crate::encoding::Unpacked<PlainBytes>)
    for type (Vec<Cow<'a, [u8]>>) including distinguished with generics ('a));

#[cfg(test)]
mod vec_u8 {
    use super::{PlainBytes, Vec};
    use crate::encoding::test::check_type_test;
    check_type_test!(PlainBytes, expedient, Vec<u8>, WireType::LengthDelimited);
    check_type_test!(
        PlainBytes,
        distinguished,
        Vec<u8>,
        WireType::LengthDelimited
    );
}

impl Wiretyped<PlainBytes> for Cow<'_, [u8]> {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}

impl ValueEncoder<PlainBytes> for Cow<'_, [u8]> {
    #[inline]
    fn encode_value<B: BufMut + ?Sized>(value: &Cow<[u8]>, buf: &mut B) {
        encode_varint(value.len() as u64, buf);
        buf.put_slice(value.as_ref());
    }

    #[inline]
    fn prepend_value<B: ReverseBuf + ?Sized>(value: &Cow<[u8]>, buf: &mut B) {
        buf.prepend_slice(value);
        prepend_varint(value.len() as u64, buf);
    }

    #[inline]
    fn value_encoded_len(value: &Cow<[u8]>) -> usize {
        encoded_len_varint(value.len() as u64) + value.len()
    }

    #[inline]
    fn decode_value<B: Buf + ?Sized>(
        value: &mut Cow<[u8]>,
        buf: Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        ValueEncoder::<PlainBytes>::decode_value(value.to_mut(), buf, ctx)
    }
}

impl DistinguishedValueEncoder<PlainBytes> for Cow<'_, [u8]> {
    #[inline]
    fn decode_value_distinguished<const ALLOW_EMPTY: bool>(
        value: &mut Cow<[u8]>,
        buf: Capped<impl Buf + ?Sized>,
        ctx: DecodeContext,
    ) -> Result<Canonicity, DecodeError> {
        DistinguishedValueEncoder::<PlainBytes>::decode_value_distinguished::<ALLOW_EMPTY>(
            value.to_mut(),
            buf,
            ctx,
        )
    }
}

#[cfg(test)]
mod cow_bytes {
    use super::{Cow, PlainBytes};
    use crate::encoding::test::check_type_test;
    check_type_test!(PlainBytes, expedient, Cow<[u8]>, WireType::LengthDelimited);
    check_type_test!(
        PlainBytes,
        distinguished,
        Cow<[u8]>,
        WireType::LengthDelimited
    );
}

impl<const N: usize> Wiretyped<PlainBytes> for [u8; N] {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}

impl<const N: usize> ValueEncoder<PlainBytes> for [u8; N] {
    #[inline]
    fn encode_value<B: BufMut + ?Sized>(value: &[u8; N], mut buf: &mut B) {
        buf.put_slice(&const_varint(N as u64));
        (&mut buf).put(value.as_slice())
    }

    #[inline]
    fn prepend_value<B: ReverseBuf + ?Sized>(value: &[u8; N], buf: &mut B) {
        buf.prepend_slice(value);
        buf.prepend_slice(&const_varint(N as u64))
    }

    #[inline]
    fn value_encoded_len(_value: &[u8; N]) -> usize {
        const_varint(N as u64).len() + N
    }

    #[inline]
    fn many_values_encoded_len<I>(values: I) -> usize
    where
        I: ExactSizeIterator,
        I::Item: Deref<Target = [u8; N]>,
    {
        values.len() * (const_varint(N as u64).len() + N)
    }

    fn decode_value<B: Buf + ?Sized>(
        value: &mut [u8; N],
        mut buf: Capped<B>,
        _ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        let mut delimited = buf.take_length_delimited()?;
        if delimited.remaining_before_cap() != N {
            return Err(DecodeError::new(InvalidValue));
        }
        delimited.copy_to_slice(value.as_mut_slice());
        Ok(())
    }
}

impl<const N: usize> DistinguishedValueEncoder<PlainBytes> for [u8; N] {
    fn decode_value_distinguished<const ALLOW_EMPTY: bool>(
        value: &mut [u8; N],
        buf: Capped<impl Buf + ?Sized>,
        ctx: DecodeContext,
    ) -> Result<Canonicity, DecodeError> {
        ValueEncoder::<PlainBytes>::decode_value(value, buf, ctx)?;
        Ok(if !ALLOW_EMPTY && value.is_empty() {
            Canonicity::NotCanonical
        } else {
            Canonicity::Canonical
        })
    }
}

#[cfg(test)]
mod u8_array {
    mod length_0 {
        use crate::encoding::test::check_type_test;
        use crate::encoding::PlainBytes;
        check_type_test!(PlainBytes, expedient, [u8; 0], WireType::LengthDelimited);
        check_type_test!(
            PlainBytes,
            distinguished,
            [u8; 0],
            WireType::LengthDelimited
        );
    }

    mod length_1 {
        use crate::encoding::test::check_type_test;
        use crate::encoding::PlainBytes;
        check_type_test!(PlainBytes, expedient, [u8; 1], WireType::LengthDelimited);
        check_type_test!(
            PlainBytes,
            distinguished,
            [u8; 1],
            WireType::LengthDelimited
        );
    }

    mod length_8 {
        use crate::encoding::test::check_type_test;
        use crate::encoding::PlainBytes;
        check_type_test!(PlainBytes, expedient, [u8; 8], WireType::LengthDelimited);
        check_type_test!(
            PlainBytes,
            distinguished,
            [u8; 8],
            WireType::LengthDelimited
        );
    }

    mod length_13 {
        use crate::encoding::test::check_type_test;
        use crate::encoding::PlainBytes;
        check_type_test!(PlainBytes, expedient, [u8; 13], WireType::LengthDelimited);
        check_type_test!(
            PlainBytes,
            distinguished,
            [u8; 13],
            WireType::LengthDelimited
        );
    }
}

#[allow(unused_macros)]
macro_rules! plain_bytes_vec_impl {
    (
        $ty:ty,
        $value:ident, $buf:ident, $chunk:ident,
        $do_reserve:expr,
        $do_extend:expr
        $(, with generics ($($generics:tt)*))?
    ) => {
        impl$(<$($generics)*>)? Wiretyped<PlainBytes> for $ty {
            const WIRE_TYPE: WireType = WireType::LengthDelimited;
        }

        impl$(<$($generics)*>)? ValueEncoder<PlainBytes> for $ty {
            fn encode_value<B: BufMut + ?Sized>(value: &$ty, buf: &mut B) {
                encode_varint(value.len() as u64, buf);
                buf.put_slice(value.as_slice());
            }

            fn prepend_value<B: ReverseBuf + ?Sized>(value: &$ty, buf: &mut B) {
                buf.prepend_slice(value);
                prepend_varint(value.len() as u64, buf);
            }

            fn value_encoded_len(value: &$ty) -> usize {
                encoded_len_varint(value.len() as u64) + value.len()
            }

            fn decode_value<B: Buf + ?Sized>(
                $value: &mut $ty,
                mut buf: Capped<B>,
                _ctx: DecodeContext,
            ) -> Result<(), DecodeError> {
                let mut $buf = buf.take_length_delimited()?.take_all();
                $value.clear();
                $do_reserve;
                while $buf.has_remaining() {
                    let $chunk = $buf.chunk();
                    $do_extend;
                    $buf.advance($chunk.len());
                }
                Ok(())
            }
        }

        impl$(<$($generics)*>)? DistinguishedValueEncoder<PlainBytes> for $ty {
            fn decode_value_distinguished<const ALLOW_EMPTY: bool>(
                value: &mut $ty,
                buf: Capped<impl Buf + ?Sized>,
                ctx: DecodeContext,
            ) -> Result<Canonicity, DecodeError> {
                ValueEncoder::<PlainBytes>::decode_value(value, buf, ctx)?;
                Ok(if !ALLOW_EMPTY && value.is_empty() {
                    Canonicity::NotCanonical
                } else {
                    Canonicity::Canonical
                })
            }
        }
    }
}

#[cfg(feature = "arrayvec")]
plain_bytes_vec_impl!(
    arrayvec::ArrayVec<u8, N>,
    value,
    buf,
    chunk,
    if buf.remaining() > N {
        return Err(DecodeError::new(InvalidValue));
    },
    value.extend(chunk.iter().cloned()),
    with generics (const N: usize)
);

#[cfg(feature = "smallvec")]
plain_bytes_vec_impl!(
    smallvec::SmallVec<A>,
    value,
    buf,
    chunk,
    value.reserve(buf.remaining()),
    value.extend_from_slice(chunk),
    with generics (A: smallvec::Array<Item = u8>)
);

#[cfg(feature = "thin-vec")]
plain_bytes_vec_impl!(
    thin_vec::ThinVec<u8>,
    value,
    buf,
    chunk,
    value.reserve(buf.remaining()),
    value.extend_from_slice(chunk)
);

#[cfg(feature = "tinyvec")]
plain_bytes_vec_impl!(
    tinyvec::ArrayVec<A>,
    value,
    buf,
    chunk,
    if buf.remaining() > A::CAPACITY {
        return Err(DecodeError::new(InvalidValue));
    },
    value.extend_from_slice(chunk),
    with generics (A: tinyvec::Array<Item = u8>)
);

#[cfg(feature = "tinyvec")]
plain_bytes_vec_impl!(
    tinyvec::TinyVec<A>,
    value,
    buf,
    chunk,
    value.reserve(buf.remaining()),
    value.extend_from_slice(chunk),
    with generics (A: tinyvec::Array<Item = u8>)
);

#[cfg(test)]
mod third_party_vecs {
    #[allow(unused_macros)]
    macro_rules! check_unbounded {
        ($ty:ty) => {
            use crate::encoding::test::check_type_test;
            use crate::encoding::PlainBytes;
            use alloc::vec::Vec;
            check_type_test!(
                PlainBytes,
                expedient,
                from Vec<u8>,
                into $ty,
                converter(val) val.into_iter().collect(),
                WireType::LengthDelimited
            );
            check_type_test!(
                PlainBytes,
                distinguished,
                from Vec<u8>,
                into $ty,
                converter(val) val.into_iter().collect(),
                WireType::LengthDelimited
            );
        }
    }
    #[allow(unused_macros)]
    macro_rules! check_bounded {
        ($ty:ty, $N:expr) => {
            use proptest::prelude::*;
            use crate::encoding::{PlainBytes, WireType};
            use crate::encoding::test::{distinguished, expedient};
            proptest! {
                #[test]
                fn check(from in prop::collection::vec(any::<u8>(), 0..=$N), tag: u32) {
                    let into: $ty = from.into_iter().collect();
                    expedient::check_type::<$ty, PlainBytes>(
                        into.clone(),
                        tag,
                        WireType::LengthDelimited,
                    )?;
                    distinguished::check_type::<$ty, PlainBytes>(
                        into,
                        tag,
                        WireType::LengthDelimited,
                    )?;
                }
                #[test]
                fn check_optional(from in prop::option::of(prop::collection::vec(any::<u8>(), 0..=$N)), tag: u32) {
                    let into: Option<$ty> = from.map(|val| val.into_iter().collect());
                    expedient::check_type::<Option<$ty>, PlainBytes>(
                        into.clone(),
                        tag,
                        WireType::LengthDelimited,
                    )?;
                    distinguished::check_type::<Option<$ty>, PlainBytes>(
                        into,
                        tag,
                        WireType::LengthDelimited,
                    )?;
                }
            }
        }
    }

    #[cfg(feature = "arrayvec")]
    mod arrayvec {
        check_bounded!(arrayvec::ArrayVec<u8, 8>, 8);
    }
    #[cfg(feature = "smallvec")]
    mod smallvec {
        check_unbounded!(smallvec::SmallVec<[u8; 8]>);
    }
    #[cfg(feature = "thin-vec")]
    mod thin_vec {
        check_unbounded!(thin_vec::ThinVec<u8>);
    }
    #[cfg(feature = "tinyvec")]
    mod tinyarrayvec {
        check_bounded!(tinyvec::ArrayVec<[u8; 8]>, 8);
    }
    #[cfg(feature = "tinyvec")]
    mod tinyvec {
        check_unbounded!(tinyvec::TinyVec<[u8; 8]>);
    }
}
