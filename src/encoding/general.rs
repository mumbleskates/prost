use alloc::borrow::Cow;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;
use core::mem;
use core::str;

use bytes::{Buf, BufMut, Bytes};

use crate::buf::ReverseBuf;
use crate::encoding::{
    delegate_encoding, delegate_value_encoding, encode_varint, encoded_len_varint,
    encoder_where_value_encoder, prepend_varint, Canonicity, Capped, DecodeContext, DecodeError,
    DistinguishedValueEncoder, Encoder, Fixed, Map, PlainBytes, Unpacked, ValueEncoder, Varint,
    WireType, Wiretyped,
};
use crate::message::{merge, merge_distinguished, RawDistinguishedMessage, RawMessage};
use crate::Blob;
use crate::DecodeErrorKind::InvalidValue;

pub struct General;

encoder_where_value_encoder!(General);

// General implements unpacked encodings by default, but only for select collection types. Other
// implementers of the `Collection` trait must use Unpacked or Packed.
delegate_encoding!(delegate from (General) to (Unpacked<General>)
    for type (Vec<T>) including distinguished with generics (T));
delegate_encoding!(delegate from (General) to (Unpacked<General>)
    for type (Cow<'a, [T]>) including distinguished
    with where clause (T: Clone)
    with generics ('a, T));
#[cfg(feature = "arrayvec")]
delegate_encoding!(delegate from (General) to (Unpacked<General>)
    for type (arrayvec::ArrayVec<T, N>) including distinguished
    with generics (T, const N: usize));
#[cfg(feature = "smallvec")]
delegate_encoding!(delegate from (General) to (Unpacked<General>)
    for type (smallvec::SmallVec<A>) including distinguished
    with where clause (A: smallvec::Array<Item = T>)
    with generics (T, A));
#[cfg(feature = "thin-vec")]
delegate_encoding!(delegate from (General) to (Unpacked<General>)
    for type (thin_vec::ThinVec<T>) including distinguished with generics (T));
#[cfg(feature = "tinyvec")]
delegate_encoding!(delegate from (General) to (Unpacked<General>)
    for type (tinyvec::ArrayVec<A>) including distinguished
    with where clause (A: tinyvec::Array<Item = T>)
    with generics (T, A));
#[cfg(feature = "tinyvec")]
delegate_encoding!(delegate from (General) to (Unpacked<General>)
    for type (tinyvec::TinyVec<A>) including distinguished
    with where clause (A: tinyvec::Array<Item = T>)
    with generics (T, A));
delegate_encoding!(delegate from (General) to (Unpacked<General>)
    for type (BTreeSet<T>) including distinguished with generics (T));
delegate_value_encoding!(delegate from (General) to (Map<General, General>)
    for type (BTreeMap<K, V>) including distinguished
    with where clause for expedient (K: Ord)
    with where clause for distinguished (V: Eq)
    with generics (K, V));
#[cfg(feature = "std")]
delegate_encoding!(delegate from (General) to (Unpacked<General>)
    for type (std::collections::HashSet<T, S>)
    with where clause (S: Default + core::hash::BuildHasher)
    with generics (T, S));
#[cfg(feature = "std")]
delegate_value_encoding!(delegate from (General) to (Map<General, General>)
    for type (std::collections::HashMap<K, V, S>)
    with where clause (K: Eq + core::hash::Hash, S: Default + core::hash::BuildHasher)
    with generics (K, V, S));
#[cfg(feature = "hashbrown")]
delegate_encoding!(delegate from (General) to (Unpacked<General>)
    for type (hashbrown::HashSet<T, S>)
    with where clause (S: Default + core::hash::BuildHasher)
    with generics (T, S));
#[cfg(feature = "hashbrown")]
delegate_value_encoding!(delegate from (General) to (Map<General, General>)
    for type (hashbrown::HashMap<K, V, S>)
    with where clause (K: Eq + core::hash::Hash, S: Default + core::hash::BuildHasher)
    with generics (K, V, S));

// General encodes bool and integers as varints.
delegate_value_encoding!(delegate from (General) to (Varint)
    for type (bool) including distinguished);
delegate_value_encoding!(delegate from (General) to (Varint)
    for type (u16) including distinguished);
delegate_value_encoding!(delegate from (General) to (Varint)
    for type (i16) including distinguished);
delegate_value_encoding!(delegate from (General) to (Varint)
    for type (u32) including distinguished);
delegate_value_encoding!(delegate from (General) to (Varint)
    for type (i32) including distinguished);
delegate_value_encoding!(delegate from (General) to (Varint)
    for type (u64) including distinguished);
delegate_value_encoding!(delegate from (General) to (Varint)
    for type (i64) including distinguished);
delegate_value_encoding!(delegate from (General) to (Varint)
    for type (usize) including distinguished);
delegate_value_encoding!(delegate from (General) to (Varint)
    for type (isize) including distinguished);

// General also encodes floating point values.
delegate_value_encoding!(delegate from (General) to (Fixed) for type (f32));
delegate_value_encoding!(delegate from (General) to (Fixed) for type (f64));

impl Wiretyped<General> for String {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}

impl ValueEncoder<General> for String {
    #[inline]
    fn encode_value<B: BufMut + ?Sized>(value: &String, buf: &mut B) {
        encode_varint(value.len() as u64, buf);
        buf.put_slice(value.as_bytes());
    }

    #[inline]
    fn prepend_value<B: ReverseBuf + ?Sized>(value: &String, buf: &mut B) {
        buf.prepend_slice(value.as_bytes());
        prepend_varint(value.len() as u64, buf);
    }

    #[inline]
    fn value_encoded_len(value: &String) -> usize {
        encoded_len_varint(value.len() as u64) + value.len()
    }

    #[inline]
    fn decode_value<B: Buf + ?Sized>(
        value: &mut String,
        mut buf: Capped<B>,
        _ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        // ## Unsafety
        //
        // Copies string data from the buffer, with an additional check of utf-8 well-formedness.
        // If the utf-8 is not well-formed, or if any other error occurs while copying the data,
        // then the string is cleared so as to avoid leaking a string field with invalid data.
        //
        // This implementation uses the unsafe `String::as_mut_vec` method instead of the safe
        // alternative of temporarily swapping an empty `String` into the field, because it results
        // in up to 10% better performance on the protobuf message decoding benchmarks.
        //
        // It's required when using `String::as_mut_vec` that invalid utf-8 data not be leaked into
        // the backing `String`. To enforce this, even in the event of a panic in the decoder or
        // in the buf implementation, a drop guard is used.
        struct DropGuard<'a>(&'a mut Vec<u8>);
        impl Drop for DropGuard<'_> {
            #[inline]
            fn drop(&mut self) {
                self.0.clear();
            }
        }

        let source = buf.take_length_delimited()?.take_all();
        // If we must copy, make sure to copy only once.
        value.clear();
        value.reserve(source.remaining());
        unsafe {
            let drop_guard = DropGuard(value.as_mut_vec());
            drop_guard.0.put(source);
            match str::from_utf8(drop_guard.0) {
                Ok(_) => {
                    // Success; do not clear the bytes.
                    mem::forget(drop_guard);
                    Ok(())
                }
                Err(_) => Err(DecodeError::new(InvalidValue)),
            }
        }
    }
}

impl DistinguishedValueEncoder<General> for String {
    const CHECKS_EMPTY: bool = false;

    #[inline]
    fn decode_value_distinguished<const ALLOW_EMPTY: bool>(
        value: &mut String,
        buf: Capped<impl Buf + ?Sized>,
        ctx: DecodeContext,
    ) -> Result<Canonicity, DecodeError> {
        Self::decode_value(value, buf, ctx)?;
        Ok(Canonicity::Canonical)
    }
}

#[cfg(test)]
mod string {
    use super::{General, String};
    use crate::encoding::test::check_type_test;
    check_type_test!(General, expedient, String, WireType::LengthDelimited);
    check_type_test!(General, distinguished, String, WireType::LengthDelimited);
}

impl Wiretyped<General> for Cow<'_, str> {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}

impl ValueEncoder<General> for Cow<'_, str> {
    #[inline]
    fn encode_value<B: BufMut + ?Sized>(value: &Cow<str>, buf: &mut B) {
        encode_varint(value.len() as u64, buf);
        buf.put_slice(value.as_bytes());
    }

    #[inline]
    fn prepend_value<B: ReverseBuf + ?Sized>(value: &Cow<str>, buf: &mut B) {
        buf.prepend_slice(value.as_bytes());
        prepend_varint(value.len() as u64, buf);
    }

    #[inline]
    fn value_encoded_len(value: &Cow<str>) -> usize {
        encoded_len_varint(value.len() as u64) + value.len()
    }

    #[inline]
    fn decode_value<B: Buf + ?Sized>(
        value: &mut Cow<str>,
        buf: Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        ValueEncoder::<General>::decode_value(value.to_mut(), buf, ctx)
    }
}

impl DistinguishedValueEncoder<General> for Cow<'_, str> {
    const CHECKS_EMPTY: bool = <String as DistinguishedValueEncoder<General>>::CHECKS_EMPTY;

    #[inline]
    fn decode_value_distinguished<const ALLOW_EMPTY: bool>(
        value: &mut Cow<str>,
        buf: Capped<impl Buf + ?Sized>,
        ctx: DecodeContext,
    ) -> Result<Canonicity, DecodeError> {
        DistinguishedValueEncoder::<General>::decode_value_distinguished::<ALLOW_EMPTY>(
            value.to_mut(),
            buf,
            ctx,
        )
    }
}

#[cfg(test)]
mod cow_string {
    use super::{Cow, General};
    use crate::encoding::test::check_type_test;
    check_type_test!(General, expedient, Cow<str>, WireType::LengthDelimited);
    check_type_test!(General, distinguished, Cow<str>, WireType::LengthDelimited);
}

#[cfg(feature = "bytestring")]
impl Wiretyped<General> for bytestring::ByteString {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}

#[cfg(feature = "bytestring")]
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

#[cfg(feature = "bytestring")]
impl DistinguishedValueEncoder<General> for bytestring::ByteString {
    const CHECKS_EMPTY: bool = false;

    #[inline]
    fn decode_value_distinguished<const ALLOW_EMPTY: bool>(
        value: &mut bytestring::ByteString,
        buf: Capped<impl Buf + ?Sized>,
        ctx: DecodeContext,
    ) -> Result<Canonicity, DecodeError> {
        Self::decode_value(value, buf, ctx)?;
        Ok(Canonicity::Canonical)
    }
}

#[cfg(feature = "bytestring")]
#[cfg(test)]
mod bytestring_string {
    use super::{General, String};
    use crate::encoding::test::check_type_test;
    check_type_test!(General, expedient, from String,
        into bytestring::ByteString, WireType::LengthDelimited);
    check_type_test!(General, distinguished, from String, into bytestring::ByteString,
        WireType::LengthDelimited);
}

#[cfg(feature = "bstr")]
impl Wiretyped<General> for bstr::BString {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}

#[cfg(feature = "bstr")]
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

#[cfg(feature = "bstr")]
impl DistinguishedValueEncoder<General> for bstr::BString {
    const CHECKS_EMPTY: bool = <Vec<u8> as DistinguishedValueEncoder<PlainBytes>>::CHECKS_EMPTY;

    #[inline(always)]
    fn decode_value_distinguished<const ALLOW_EMPTY: bool>(
        value: &mut Self,
        buf: Capped<impl Buf + ?Sized>,
        ctx: DecodeContext,
    ) -> Result<Canonicity, DecodeError> {
        DistinguishedValueEncoder::<PlainBytes>::decode_value_distinguished::<ALLOW_EMPTY>(
            &mut **value,
            buf,
            ctx,
        )
    }
}

#[cfg(feature = "bstr")]
#[cfg(test)]
mod bstr_string {
    use super::{General, Vec};
    use crate::encoding::test::check_type_test;
    check_type_test!(General, expedient, from Vec<u8>, into bstr::BString,
        WireType::LengthDelimited);
    check_type_test!(General, distinguished, from Vec<u8>, into bstr::BString,
        WireType::LengthDelimited);
}

impl Wiretyped<General> for Bytes {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}

impl ValueEncoder<General> for Bytes {
    #[inline]
    fn encode_value<B: BufMut + ?Sized>(value: &Bytes, mut buf: &mut B) {
        encode_varint(value.len() as u64, buf);
        (&mut buf).put(value.clone()); // `put` needs Self to be sized, so we use the ref type
    }

    #[inline]
    fn prepend_value<B: ReverseBuf + ?Sized>(value: &Bytes, buf: &mut B) {
        buf.prepend_slice(value);
        prepend_varint(value.len() as u64, buf);
    }

    #[inline]
    fn value_encoded_len(value: &Bytes) -> usize {
        encoded_len_varint(value.len() as u64) + value.len()
    }

    #[inline]
    fn decode_value<B: Buf + ?Sized>(
        value: &mut Bytes,
        mut buf: Capped<B>,
        _ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        let mut buf = buf.take_length_delimited()?;
        let len = buf.remaining_before_cap();
        *value = buf.copy_to_bytes(len);
        Ok(())
    }
}

impl DistinguishedValueEncoder<General> for Bytes {
    const CHECKS_EMPTY: bool = false;

    #[inline]
    fn decode_value_distinguished<const ALLOW_EMPTY: bool>(
        value: &mut Bytes,
        buf: Capped<impl Buf + ?Sized>,
        ctx: DecodeContext,
    ) -> Result<Canonicity, DecodeError> {
        Self::decode_value(value, buf, ctx)?;
        Ok(Canonicity::Canonical)
    }
}

#[cfg(test)]
mod bytes_blob {
    use super::{Bytes, General, Vec};
    use crate::encoding::test::check_type_test;
    check_type_test!(General, expedient, from Vec<u8>, into Bytes, WireType::LengthDelimited);
    check_type_test!(General, distinguished, from Vec<u8>, into Bytes, WireType::LengthDelimited);
}

impl Wiretyped<General> for Blob {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}

impl ValueEncoder<General> for Blob {
    #[inline]
    fn encode_value<B: BufMut + ?Sized>(value: &Blob, buf: &mut B) {
        ValueEncoder::<PlainBytes>::encode_value(&**value, buf)
    }

    #[inline]
    fn prepend_value<B: ReverseBuf + ?Sized>(value: &Blob, buf: &mut B) {
        buf.prepend_slice(value);
        prepend_varint(value.len() as u64, buf);
    }

    #[inline]
    fn value_encoded_len(value: &Blob) -> usize {
        ValueEncoder::<PlainBytes>::value_encoded_len(&**value)
    }

    #[inline]
    fn decode_value<B: Buf + ?Sized>(
        value: &mut Blob,
        buf: Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        ValueEncoder::<PlainBytes>::decode_value(&mut **value, buf, ctx)
    }
}

impl DistinguishedValueEncoder<General> for Blob {
    const CHECKS_EMPTY: bool = <Vec<u8> as DistinguishedValueEncoder<PlainBytes>>::CHECKS_EMPTY;

    #[inline]
    fn decode_value_distinguished<const ALLOW_EMPTY: bool>(
        value: &mut Blob,
        buf: Capped<impl Buf + ?Sized>,
        ctx: DecodeContext,
    ) -> Result<Canonicity, DecodeError> {
        DistinguishedValueEncoder::<PlainBytes>::decode_value_distinguished::<ALLOW_EMPTY>(
            &mut **value,
            buf,
            ctx,
        )
    }
}

#[cfg(test)]
mod blob {
    use super::{Blob, General};
    use crate::encoding::test::check_type_test;
    check_type_test!(General, expedient, Blob, WireType::LengthDelimited);
    check_type_test!(General, distinguished, Blob, WireType::LengthDelimited);
}

// TODO(widders): time, chrono, std::time support
//
// deps this may create:
//  * tinyvec, for tinyvec's array vec
//      * change tinyvec dependency to not have the alloc feature by default
//      * the tinyvec support feature should enable this feature as well instead
//  * derive
//
// crate time: (other deps: derive)
//  * struct Date
//      * store as [year, ordinal-zero] (packed<varint> with trailing zeros removed)
//  * struct Time
//      * store as [hour, minute, second, nanos] (packed<varint> with trailing zeros removed)
//  * struct PrimitiveDateTime
//      * aggregate of (Date, Time)
//      * store as [year, ordinal-zero, hour, minute, second, nanos]
//        (packed<varint> with trailing zeros removed)
//  * struct UtcOffset
//      * store as [hour, minute, second] (packed<varint> with trailing zeros removed)
//  * struct OffsetDateTime
//      * aggregate of (PrimitiveDateTime, UtcOffset)
//      * store as tuple
//  * struct Duration
//      * matches bilrost_types::Duration
//      * use derived storage
//
// crate chrono: (other deps: derive)
//  * struct NaiveDate
//      * store as [year, ordinal-zero] (packed<varint> with trailing zeros removed)
//  * struct NaiveTime
//      * store as [hour, minute, second, nanos] (packed<varint> with trailing zeros removed)
//  * struct NaiveDateTime
//      * aggregate of (NaiveDate, NaiveTime)
//      * store as [year, ordinal-zero, hour, minute, second, nanos]
//        (packed<varint> with trailing zeros removed)
//  * trait TimeZone
//      * has an Offset trait associated type that's stored with aware times. we need to be able to
//        encode these
//      * Utc: ()
//      * FixedOffset: [hour, minute, second] (packed<varint> with trailing zeros removed)
//      * Local: maybe don't support this one
//      * there is also crate chrono-tz, but it doesn't(?) make sense to support that. concerns
//        involving the shifting sands of timezone definitions are outside the responsibilities of
//        an encoding library (maybe we can just check it and make it non-canonical? these types are
//        probably all non-canonical anyway)
//  * struct Date<impl TimeZone>
//      * aggregate of (NaiveDate, offset)
//      * store as tuple
//  * struct DateTime<impl TimeZone>
//      * aggreagate of (NaiveDateTime, offset)
//      * store as tuple
//  * struct TimeDelta
//      * matches bilrost_types::Duration, but nanos is always positive
//      * use derived storage
//
// std::time: (deps: none)
//  * struct Duration (this is actually in core)
//      * unsigned duration type of u64 seconds plus nanos, available via .as_secs() and
//        .subsec_nanos()
//      * store as [seconds, nanos] (packed<varint> with trailing zeros removed)
//  * struct SystemTime
//      * we must measure this via Ord against std::time::UNIX_EPOCH and subtract the greater to
//        get a Duration of the magnitude
//      * the seconds portion is effectively up a 65 bit one's complement value, which is difficult
//      * store as:
//          * epoch: empty
//          * greater: ['+' as u64, seconds, nanos] (packed<varint> with trailing zero(?) removed)
//          * lesser: ['-' as u64, seconds, nanos] (packed<varint> with trailing zero(?) removed)

mod impl_core_time_duration {
    use super::*;
    use crate::encoding::proxy_encoder;
    use crate::DecodeErrorKind;

    type Proxy = crate::encoding::local_proxy::LocalProxy<u64, 2>;
    type Encoder = crate::encoding::Packed<Varint>;

    fn empty_proxy() -> Proxy {
        Proxy::new_empty()
    }

    fn to_proxy(from: &core::time::Duration) -> Proxy {
        Proxy::new_without_empty_suffix([from.as_secs(), from.subsec_nanos() as u64])
    }

    fn from_proxy(proxy: Proxy) -> Result<core::time::Duration, DecodeErrorKind> {
        let [secs, nanos] = proxy.into_inner();
        nanos
            .try_into()
            .map_err(|_| InvalidValue)
            .and_then(|nanos| {
                if nanos > 999_999_999 {
                    Err(InvalidValue)
                } else {
                    Ok(core::time::Duration::new(secs, nanos))
                }
            })
    }

    fn from_proxy_distinguished(
        proxy: Proxy,
    ) -> Result<(core::time::Duration, Canonicity), DecodeErrorKind> {
        let ([secs, nanos], canon) = proxy.into_inner_distinguished();
        nanos
            .try_into()
            .map_err(|_| InvalidValue)
            .and_then(|nanos| {
                if nanos > 999_999_999 {
                    Err(InvalidValue)
                } else {
                    Ok((core::time::Duration::new(secs, nanos), canon))
                }
            })
    }

    proxy_encoder!(
        encode type (core::time::Duration) with encoder (General)
        via proxy (Proxy) using real encoder (Encoder)
        including distinguished
    );

    #[cfg(test)]
    mod test {
        use super::*;
        use crate::encoding::test::{check_type_empty, check_type_test};

        check_type_empty!(core::time::Duration, via proxy Proxy);
        check_type_test!(
            General,
            expedient,
            core::time::Duration,
            WireType::LengthDelimited
        );
        check_type_empty!(core::time::Duration, via distinguished proxy Proxy);
        check_type_test!(
            General,
            distinguished,
            core::time::Duration,
            WireType::LengthDelimited
        );
    }
}

#[cfg(feature = "std")]
mod impl_std_time_systemtime {
    use super::*;
    use crate::encoding::proxy_encoder;
    use crate::DecodeErrorKind::{self, OutOfDomainValue};
    use std::cmp::Ordering;
    use std::time::{SystemTime, UNIX_EPOCH};

    type Proxy = crate::encoding::local_proxy::LocalProxy<u64, 3>;
    type Encoder = crate::encoding::Packed<Varint>;

    fn empty_proxy() -> Proxy {
        Proxy::new_empty()
    }

    fn to_proxy(from: &SystemTime) -> Proxy {
        let (symbol, small, big) = match from.cmp(&UNIX_EPOCH) {
            Ordering::Equal => {
                return Proxy::new_empty();
            }
            Ordering::Greater => ('+', &UNIX_EPOCH, from),
            Ordering::Less => ('-', from, &UNIX_EPOCH),
        };
        let magnitude = big
            .duration_since(*small)
            .expect("SystemTime dates ordered wrong");
        Proxy::new_without_empty_suffix([
            symbol as u64,
            magnitude.as_secs(),
            magnitude.subsec_nanos() as u64,
        ])
    }

    fn from_proxy(proxy: Proxy) -> Result<SystemTime, DecodeErrorKind> {
        let (operation, secs, nanos): (fn(_, _) -> _, u64, u64) = match proxy.into_inner() {
            [0, 0, 0] => return Ok(UNIX_EPOCH),
            [symbol, secs, nanos] if symbol == '+' as u64 => (SystemTime::checked_add, secs, nanos),
            [symbol, secs, nanos] if symbol == '-' as u64 => (SystemTime::checked_sub, secs, nanos),
            _ => return Err(InvalidValue),
        };
        let nanos = nanos
            .try_into()
            .map_err(|_| InvalidValue)
            .and_then(|nanos| {
                if nanos > 999_999_999 {
                    Err(InvalidValue)
                } else {
                    Ok(nanos)
                }
            })?;
        operation(&UNIX_EPOCH, core::time::Duration::new(secs, nanos)).ok_or(OutOfDomainValue)
    }

    // SystemTime does not have a distinguished decoding because the implementations vary enough
    // from platform to platform, including by their accuracy, that it isn't worthwhile to validate
    // its canonicity at the encoding level; if we did, values still might not even round trip. If
    // that kind of guarantee is needed, a dedicated stable time struct type should be used.
    proxy_encoder!(
        encode type (SystemTime) with encoder (General)
        via proxy (Proxy) using real encoder (Encoder)
    );

    #[cfg(test)]
    mod test {
        use super::*;
        use crate::encoding::test::{check_type_empty, check_type_test};

        check_type_empty!(SystemTime, via proxy Proxy);
        check_type_test!(General, expedient, SystemTime, WireType::LengthDelimited);
    }
}

#[cfg(feature = "chrono")]
mod impl_chrono {
    use super::*;
    mod naivedate {
        use super::*;
        use crate::encoding::proxy_encoder;
        use crate::DecodeErrorKind::{self, OutOfDomainValue};
        use chrono::{Datelike, NaiveDate};

        type Proxy = crate::encoding::local_proxy::LocalProxy<i32, 2>;
        type Encoder = crate::encoding::Packed<Varint>;

        fn empty_proxy() -> Proxy {
            Proxy::new_empty()
        }

        fn to_proxy(from: &NaiveDate) -> Proxy {
            Proxy::new_without_empty_suffix([from.year(), from.ordinal0() as i32])
        }

        fn from_proxy(proxy: Proxy) -> Result<NaiveDate, DecodeErrorKind> {
            let [year, ordinal0] = proxy.into_inner();
            let ordinal0: u32 = ordinal0.try_into().map_err(|_| InvalidValue)?;
            NaiveDate::from_yo_opt(year, ordinal0 + 1).ok_or(OutOfDomainValue)
        }

        fn from_proxy_distinguished(
            proxy: Proxy,
        ) -> Result<(NaiveDate, Canonicity), DecodeErrorKind> {
            let ([year, ordinal0], canon) = proxy.into_inner_distinguished();
            let ordinal0: u32 = ordinal0.try_into().map_err(|_| InvalidValue)?;
            NaiveDate::from_yo_opt(year, ordinal0 + 1)
                .map(|date| (date, canon))
                .ok_or(OutOfDomainValue)
        }

        proxy_encoder!(
            encode type (NaiveDate) with encoder (General)
            via proxy (Proxy) using real encoder (Encoder)
            including distinguished
        );

        #[cfg(test)]
        mod test {
            use super::*;
            use crate::encoding::test::{check_type_empty, check_type_test};

            check_type_empty!(NaiveDate, via proxy Proxy);
            check_type_test!(
                General,
                expedient,
                from Vec<u8>,
                into NaiveDate,
                converter(b) {
                    use arbitrary::{Arbitrary, Unstructured};
                    NaiveDate::arbitrary(&mut Unstructured::new(&b)).unwrap()
                },
                WireType::LengthDelimited
            );
            check_type_empty!(NaiveDate, via distinguished proxy Proxy);
            check_type_test!(
                General,
                distinguished,
                from Vec<u8>,
                into NaiveDate,
                converter(b) {
                    use arbitrary::{Arbitrary, Unstructured};
                    NaiveDate::arbitrary(&mut Unstructured::new(&b)).unwrap()
                },
                WireType::LengthDelimited
            );
        }
    }
}

impl<T> Wiretyped<General> for T
where
    T: RawMessage,
{
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}

impl<T> ValueEncoder<General> for T
where
    T: RawMessage,
{
    #[inline]
    fn encode_value<B: BufMut + ?Sized>(value: &T, buf: &mut B) {
        encode_varint(value.raw_encoded_len() as u64, buf);
        value.raw_encode(buf);
    }

    #[inline]
    fn prepend_value<B: ReverseBuf + ?Sized>(value: &T, buf: &mut B) {
        let end = buf.remaining();
        value.raw_prepend(buf);
        prepend_varint((buf.remaining() - end) as u64, buf);
    }

    #[inline]
    fn value_encoded_len(value: &T) -> usize {
        let inner_len = value.raw_encoded_len();
        encoded_len_varint(inner_len as u64) + inner_len
    }

    #[inline]
    fn decode_value<B: Buf + ?Sized>(
        value: &mut T,
        mut buf: Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        ctx.limit_reached()?;
        merge(value, buf.take_length_delimited()?, ctx.enter_recursion())
    }
}

impl<T> DistinguishedValueEncoder<General> for T
where
    T: RawDistinguishedMessage + Eq,
{
    const CHECKS_EMPTY: bool = true; // Empty messages are always zero-length

    #[inline]
    fn decode_value_distinguished<const ALLOW_EMPTY: bool>(
        value: &mut T,
        mut buf: Capped<impl Buf + ?Sized>,
        ctx: DecodeContext,
    ) -> Result<Canonicity, DecodeError> {
        ctx.limit_reached()?;
        let buf = buf.take_length_delimited()?;
        // Empty message types always encode and decode from zero bytes. It is far cheaper to check
        // here than to check after the value has been decoded and checking the message's
        // `is_empty()`.
        if !ALLOW_EMPTY && buf.remaining_before_cap() == 0 {
            return Ok(Canonicity::NotCanonical);
        }
        merge_distinguished(value, buf, ctx.enter_recursion())
    }
}
