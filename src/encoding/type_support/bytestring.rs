use crate::buf::ReverseBuf;
use crate::encoding::value_traits::for_overwrite_via_default;
use crate::encoding::{encode_varint, encoded_len_varint, prepend_varint, Capped, DecodeContext, DistinguishedValueEncoder, EmptyState, General, RestrictedDecodeContext, ValueEncoder, WireType, Wiretyped};
use crate::DecodeErrorKind::InvalidValue;
use crate::{Canonicity, DecodeError};
use bytes::{Buf, BufMut};

for_overwrite_via_default!(bytestring::ByteString);

impl EmptyState for bytestring::ByteString {
    #[inline]
    fn is_empty(&self) -> bool {
        str::is_empty(self)
    }

    #[inline]
    fn clear(&mut self) {
        *self = Self::empty();
    }
}

impl Wiretyped<General> for bytestring::ByteString {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}

impl ValueEncoder<General> for bytestring::ByteString {
    #[inline]
    fn encode_value<B: BufMut + ?Sized>(value: &bytestring::ByteString, buf: &mut B) {
        encode_varint(value.len() as u64, buf);
        buf.put_slice(value.as_bytes());
    }

    #[inline]
    fn prepend_value<B: ReverseBuf + ?Sized>(value: &bytestring::ByteString, buf: &mut B) {
        buf.prepend_slice(value.as_bytes());
        prepend_varint(value.len() as u64, buf);
    }

    #[inline]
    fn value_encoded_len(value: &bytestring::ByteString) -> usize {
        encoded_len_varint(value.len() as u64) + value.len()
    }

    #[inline]
    fn decode_value<B: Buf + ?Sized>(
        value: &mut bytestring::ByteString,
        mut buf: Capped<B>,
        _ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        let mut string_data = buf.take_length_delimited()?;
        let string_len = string_data.remaining_before_cap();
        *value = bytestring::ByteString::try_from(string_data.copy_to_bytes(string_len))
            .map_err(|_| DecodeError::new(InvalidValue))?;
        Ok(())
    }
}

impl DistinguishedValueEncoder<General> for bytestring::ByteString {
    const CHECKS_EMPTY: bool = false;

    #[inline]
    fn decode_value_distinguished<const ALLOW_EMPTY: bool>(
        value: &mut bytestring::ByteString,
        buf: Capped<impl Buf + ?Sized>,
        ctx: RestrictedDecodeContext,
    ) -> Result<Canonicity, DecodeError> {
        Self::decode_value(value, buf, ctx.expedient_context())?;
        Ok(Canonicity::Canonical)
    }
}

#[cfg(test)]
mod test {
    use super::General;
    use crate::encoding::test::check_type_test;
    use alloc::string::String;
    check_type_test!(General, expedient, from String,
        into bytestring::ByteString, WireType::LengthDelimited);
    check_type_test!(General, distinguished, from String, into bytestring::ByteString,
        WireType::LengthDelimited);
}
