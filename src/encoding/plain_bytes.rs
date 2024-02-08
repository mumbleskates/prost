use alloc::borrow::Cow;
use alloc::vec::Vec;

use bytes::{Buf, BufMut};

use crate::encoding::{
    delegate_encoding, encode_varint, encoded_len_varint, Capped, DecodeContext,
    DistinguishedEncoder, DistinguishedFieldEncoder, DistinguishedValueEncoder, Encoder,
    FieldEncoder, HasEmptyState, TagMeasurer, TagWriter, ValueEncoder, WireType, Wiretyped,
};
use crate::DecodeError;
use crate::DecodeErrorKind::{NotCanonical, UnexpectedlyRepeated};

/// `PlainBytes` implements encoding for blob values directly into `Vec<u8>`, and provides the base
/// implementation for that functionality. `Vec<u8>` cannot generically dispatch to `General`'s
/// encoding, since `General` already generically implements encoding for other kinds of `Vec`, but
/// this encoder can be used instead if it's desirable to have a value whose type is exactly
/// `Vec<u8>`.
pub struct PlainBytes;

impl<T> Encoder<T> for PlainBytes
where
    PlainBytes: ValueEncoder<T>,
    T: HasEmptyState,
{
    #[inline]
    fn encode<B: BufMut + ?Sized>(tag: u32, value: &T, buf: &mut B, tw: &mut TagWriter) {
        if !value.is_empty() {
            Self::encode_field(tag, value, buf, tw);
        }
    }

    #[inline]
    fn encoded_len(tag: u32, value: &T, tm: &mut TagMeasurer) -> usize {
        if !value.is_empty() {
            Self::field_encoded_len(tag, value, tm)
        } else {
            0
        }
    }

    #[inline]
    fn decode<B: Buf + ?Sized>(
        wire_type: WireType,
        duplicated: bool,
        value: &mut T,
        buf: Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        if duplicated {
            return Err(DecodeError::new(UnexpectedlyRepeated));
        }
        Self::decode_field(wire_type, value, buf, ctx)
    }
}

impl<T> DistinguishedEncoder<T> for PlainBytes
where
    PlainBytes: DistinguishedValueEncoder<T> + Encoder<T>,
    T: Eq + HasEmptyState,
{
    #[inline]
    fn decode_distinguished<B: Buf + ?Sized>(
        wire_type: WireType,
        duplicated: bool,
        value: &mut T,
        buf: Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        if duplicated {
            return Err(DecodeError::new(UnexpectedlyRepeated));
        }
        Self::decode_field_distinguished(wire_type, value, buf, ctx)?;
        if value.is_empty() {
            return Err(DecodeError::new(NotCanonical));
        }
        Ok(())
    }
}

impl Wiretyped<Vec<u8>> for PlainBytes {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}

impl ValueEncoder<Vec<u8>> for PlainBytes {
    fn encode_value<B: BufMut + ?Sized>(value: &Vec<u8>, buf: &mut B) {
        encode_varint(value.len() as u64, buf);
        buf.put_slice(value.as_slice());
    }

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

impl DistinguishedValueEncoder<Vec<u8>> for PlainBytes {
    fn decode_value_distinguished<B: Buf + ?Sized>(
        value: &mut Vec<u8>,
        buf: Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        Self::decode_value(value, buf, ctx)
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

impl Wiretyped<Cow<'_, [u8]>> for PlainBytes {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}

impl ValueEncoder<Cow<'_, [u8]>> for PlainBytes {
    #[inline]
    fn encode_value<B: BufMut + ?Sized>(value: &Cow<[u8]>, buf: &mut B) {
        encode_varint(value.len() as u64, buf);
        buf.put_slice(value.as_ref());
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
        Self::decode_value(value.to_mut(), buf, ctx)
    }
}

impl DistinguishedValueEncoder<Cow<'_, [u8]>> for PlainBytes {
    #[inline]
    fn decode_value_distinguished<B: Buf + ?Sized>(
        value: &mut Cow<[u8]>,
        buf: Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        Self::decode_value_distinguished(value.to_mut(), buf, ctx)
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