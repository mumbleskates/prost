use crate::buf::ReverseBuf;
use crate::encoding::value_traits::for_overwrite_via_default;
use crate::encoding::{
    Capped, DecodeContext, DistinguishedValueEncoder, EmptyState, General, PlainBytes,
    RestrictedDecodeContext, ValueEncoder, WireType, Wiretyped,
};
use crate::{Canonicity, DecodeError};
use alloc::vec::Vec;
use bytes::{Buf, BufMut};

for_overwrite_via_default!(bstr::BString);

impl EmptyState for bstr::BString {
    #[inline]
    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }

    #[inline]
    fn clear(&mut self) {
        Vec::clear(self)
    }
}

impl Wiretyped<General> for bstr::BString {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}

impl ValueEncoder<General> for bstr::BString {
    #[inline(always)]
    fn encode_value<B: BufMut + ?Sized>(value: &bstr::BString, buf: &mut B) {
        ValueEncoder::<PlainBytes>::encode_value(&**value, buf);
    }

    #[inline(always)]
    fn prepend_value<B: ReverseBuf + ?Sized>(value: &bstr::BString, buf: &mut B) {
        ValueEncoder::<PlainBytes>::prepend_value(&**value, buf);
    }

    #[inline(always)]
    fn value_encoded_len(value: &bstr::BString) -> usize {
        ValueEncoder::<PlainBytes>::value_encoded_len(&**value)
    }

    #[inline(always)]
    fn decode_value<B: Buf + ?Sized>(
        value: &mut bstr::BString,
        buf: Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        ValueEncoder::<PlainBytes>::decode_value(&mut **value, buf, ctx)
    }
}

impl DistinguishedValueEncoder<General> for bstr::BString {
    const CHECKS_EMPTY: bool = <Vec<u8> as DistinguishedValueEncoder<PlainBytes>>::CHECKS_EMPTY;

    #[inline(always)]
    fn decode_value_distinguished<const ALLOW_EMPTY: bool>(
        value: &mut Self,
        buf: Capped<impl Buf + ?Sized>,
        ctx: RestrictedDecodeContext,
    ) -> Result<Canonicity, DecodeError> {
        DistinguishedValueEncoder::<PlainBytes>::decode_value_distinguished::<ALLOW_EMPTY>(
            &mut **value,
            buf,
            ctx,
        )
    }
}

#[cfg(test)]
mod test {
    use super::{General, Vec};
    use crate::encoding::test::check_type_test;
    check_type_test!(General, expedient, from Vec<u8>, into bstr::BString,
        WireType::LengthDelimited);
    check_type_test!(General, distinguished, from Vec<u8>, into bstr::BString,
        WireType::LengthDelimited);
}
