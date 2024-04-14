use bytes::{Buf, BufMut};

use crate::buf::ReverseBuf;
use crate::encoding::value_traits::{
    Collection, DistinguishedCollection, EmptyState, NewForOverwrite,
};
use crate::encoding::{encode_varint, encoded_len_varint, prepend_varint, unpacked, Canonicity, Capped, DecodeContext, DecodeError, DistinguishedEncoder, DistinguishedValueEncoder, Encoder, FieldEncoder, General, TagMeasurer, TagRevWriter, TagWriter, ValueEncoder, WireType, Wiretyped, check_wire_type};
use crate::DecodeErrorKind::{InvalidValue, Truncated, UnexpectedlyRepeated};

pub struct Packed<E = General>(E);

/// Packed encodings are always length delimited.
impl<T, E> Wiretyped<Packed<E>> for T {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}

impl<C, T, E> ValueEncoder<Packed<E>> for C
where
    C: Collection<Item = T>,
    T: NewForOverwrite + ValueEncoder<E>,
{
    fn encode_value<B: BufMut + ?Sized>(value: &C, buf: &mut B) {
        encode_varint(
            ValueEncoder::<E>::many_values_encoded_len(value.iter()) as u64,
            buf,
        );
        for val in value.iter() {
            ValueEncoder::<E>::encode_value(val, buf);
        }
    }

    fn prepend_value<B: ReverseBuf + ?Sized>(value: &Self, buf: &mut B) {
        let end = buf.remaining();
        for val in value.reversed() {
            <T as ValueEncoder<E>>::prepend_value(val, buf);
        }
        prepend_varint((buf.remaining() - end) as u64, buf);
    }

    fn value_encoded_len(value: &C) -> usize {
        let inner_len = ValueEncoder::<E>::many_values_encoded_len(value.iter());
        // TODO(widders): address general cases where u64 may overflow usize, with care
        encoded_len_varint(inner_len as u64) + inner_len
    }

    fn decode_value<B: Buf + ?Sized>(
        value: &mut C,
        mut buf: Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        let mut capped = buf.take_length_delimited()?;
        // MSRV: this could be .is_some_and(..)
        if matches!(
            <T as Wiretyped<E>>::WIRE_TYPE.fixed_size(),
            Some(fixed_size) if capped.remaining_before_cap() % fixed_size != 0
        ) {
            // No number of fixed-sized values can pack evenly into this size.
            return Err(DecodeError::new(Truncated));
        }
        while capped.has_remaining()? {
            let mut new_val = T::new_for_overwrite();
            ValueEncoder::<E>::decode_value(&mut new_val, capped.lend(), ctx.clone())?;
            value.insert(new_val)?;
        }
        Ok(())
    }
}

impl<C, T, E> DistinguishedValueEncoder<Packed<E>> for C
where
    C: DistinguishedCollection<Item = T> + Eq,
    T: NewForOverwrite + Eq + DistinguishedValueEncoder<E>,
{
    fn decode_value_distinguished<const ALLOW_EMPTY: bool>(
        value: &mut C,
        mut buf: Capped<impl Buf + ?Sized>,
        ctx: DecodeContext,
    ) -> Result<Canonicity, DecodeError> {
        let mut capped = buf.take_length_delimited()?;
        if !ALLOW_EMPTY && capped.remaining_before_cap() == 0 {
            return Ok(Canonicity::NotCanonical);
        }
        // MSRV: this could be .is_some_and(..)
        if matches!(
            <T as Wiretyped<E>>::WIRE_TYPE.fixed_size(),
            Some(fixed_size) if capped.remaining_before_cap() % fixed_size != 0
        ) {
            // No number of fixed-sized values can pack evenly into this size.
            return Err(DecodeError::new(Truncated));
        }
        let mut canon = Canonicity::Canonical;
        while capped.has_remaining()? {
            let mut new_val = T::new_for_overwrite();
            canon.update(
                DistinguishedValueEncoder::<E>::decode_value_distinguished::<true>(
                    &mut new_val,
                    capped.lend(),
                    ctx.clone(),
                )?,
            );
            canon.update(value.insert_distinguished(new_val)?);
        }
        Ok(canon)
    }
}

/// ValueEncoder for packed repeated encodings lets this value type nest.
impl<C, T, E> Encoder<Packed<E>> for C
where
    C: Collection<Item = T> + ValueEncoder<Packed<E>>,
    T: NewForOverwrite + ValueEncoder<E>,
{
    #[inline]
    fn encode<B: BufMut + ?Sized>(tag: u32, value: &C, buf: &mut B, tw: &mut TagWriter) {
        if !value.is_empty() {
            Self::encode_field(tag, value, buf, tw);
        }
    }

    #[inline]
    fn prepend_encode<B: ReverseBuf + ?Sized>(
        tag: u32,
        value: &Self,
        buf: &mut B,
        tw: &mut TagRevWriter,
    ) {
        if !value.is_empty() {
            Self::prepend_field(tag, value, buf, tw);
        }
    }

    #[inline]
    fn encoded_len(tag: u32, value: &C, tm: &mut impl TagMeasurer) -> usize {
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
        value: &mut C,
        buf: Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        if duplicated {
            return Err(DecodeError::new(UnexpectedlyRepeated));
        }
        if wire_type == WireType::LengthDelimited {
            // We've encountered the expected length-delimited type: decode it in packed format.
            Self::decode_value(value, buf, ctx)
        } else {
            // Otherwise, try decoding it in the unpacked representation
            unpacked::decode::<C, E>(wire_type, value, buf, ctx)
        }
    }
}

impl<C, T, E> DistinguishedEncoder<Packed<E>> for C
where
    C: DistinguishedCollection<Item = T> + DistinguishedValueEncoder<Packed<E>>,
    T: NewForOverwrite + Eq + ValueEncoder<E>,
{
    #[inline]
    fn decode_distinguished<B: Buf + ?Sized>(
        wire_type: WireType,
        duplicated: bool,
        value: &mut C,
        buf: Capped<B>,
        ctx: DecodeContext,
    ) -> Result<Canonicity, DecodeError> {
        if duplicated {
            return Err(DecodeError::new(UnexpectedlyRepeated));
        }
        if wire_type == WireType::LengthDelimited {
            // We've encountered the expected length-delimited type: decode it in packed format.
            // Set ALLOW_EMPTY to false: empty collections are not canonical
            DistinguishedValueEncoder::<Packed<E>>::decode_value_distinguished::<false>(
                value, buf, ctx,
            )
        } else {
            // Otherwise, try decoding it in the unpacked representation
            unpacked::decode::<C, E>(wire_type, value, buf, ctx)?;
            Ok(Canonicity::NotCanonical)
        }
    }
}

impl<T, const N: usize, E> ValueEncoder<Packed<E>> for [T; N]
where
    T: NewForOverwrite + ValueEncoder<E>,
{
    fn encode_value<B: BufMut + ?Sized>(value: &[T; N], buf: &mut B) {
        encode_varint(
            ValueEncoder::<E>::many_values_encoded_len(value.iter()) as u64,
            buf,
        );
        for val in value.iter() {
            ValueEncoder::<E>::encode_value(val, buf);
        }
    }

    fn prepend_value<B: ReverseBuf + ?Sized>(value: &[T; N], buf: &mut B) {
        let end = buf.remaining();
        for val in value.iter().rev() {
            <T as ValueEncoder<E>>::prepend_value(val, buf);
        }
        prepend_varint((buf.remaining() - end) as u64, buf);
    }

    fn value_encoded_len(value: &[T; N]) -> usize {
        let inner_len = ValueEncoder::<E>::many_values_encoded_len(value.iter());
        // TODO(widders): address general cases where u64 may overflow usize, with care
        encoded_len_varint(inner_len as u64) + inner_len
    }

    fn decode_value<B: Buf + ?Sized>(
        value: &mut [T; N],
        mut buf: Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        let mut capped = buf.take_length_delimited()?;
        // MSRV: this could be .is_some_and(..)
        if matches!(
            <T as Wiretyped<E>>::WIRE_TYPE.fixed_size(),
            Some(fixed_size) if capped.remaining_before_cap() != fixed_size * N
        ) {
            // We know the exact size of a valid value and this isn't it.
            return Err(DecodeError::new(InvalidValue));
        }
        let mut i = 0;
        while capped.has_remaining()? {
            if i >= N {
                return Err(DecodeError::new(InvalidValue));
            }
            ValueEncoder::<E>::decode_value(&mut value[i], capped.lend(), ctx.clone())?;
            i += 1;
        }
        if i < N {
            Err(DecodeError::new(InvalidValue))
        } else {
            Ok(())
        }
    }
}

// TODO(widders): distinguished value

impl<T, const N: usize, E> Encoder<Packed<E>> for [T; N]
where
    T: EmptyState + ValueEncoder<E>,
{
    #[inline]
    fn encode<B: BufMut + ?Sized>(tag: u32, value: &[T; N], buf: &mut B, tw: &mut TagWriter) {
        if !value.is_empty() {
            <[T; N]>::encode_field(tag, value, buf, tw);
        }
    }

    #[inline]
    fn prepend_encode<B: ReverseBuf + ?Sized>(
        tag: u32,
        value: &[T; N],
        buf: &mut B,
        tw: &mut TagRevWriter,
    ) {
        if !value.is_empty() {
            <[T; N]>::prepend_field(tag, value, buf, tw);
        }
    }

    #[inline]
    fn encoded_len(tag: u32, value: &[T; N], tm: &mut impl TagMeasurer) -> usize {
        if !value.is_empty() {
            <[T; N]>::field_encoded_len(tag, value, tm)
        } else {
            0
        }
    }

    #[inline]
    fn decode<B: Buf + ?Sized>(
        wire_type: WireType,
        duplicated: bool,
        value: &mut [T; N],
        buf: Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        if duplicated {
            return Err(DecodeError::new(UnexpectedlyRepeated));
        }
        check_wire_type(WireType::LengthDelimited, wire_type);
        // TODO(widders): cross-wire with corresponding unpacked
        // if wire_type == WireType::LengthDelimited {
        //     // We've encountered the expected length-delimited type: decode it in packed format.
            Self::decode_value(value, buf, ctx)
        // } else {
        //     // Otherwise, try decoding it in the unpacked representation
        //     unpacked::decode::<<[T; N]>, E>(wire_type, value, buf, ctx)
        // }
    }
}

// TODO(widders): distinguished encode
