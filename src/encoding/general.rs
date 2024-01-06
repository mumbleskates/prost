use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;
use core::mem;
use core::str;
#[cfg(feature = "std")]
use std::collections::{HashMap, HashSet};

use crate::encoding::{
    check_wire_type, delegate_encoding, delegate_value_encoding, encode_varint, encoded_len_varint,
    Capped, DecodeContext, DistinguishedEncoder, DistinguishedFieldEncoder,
    DistinguishedValueEncoder, Encoder, FieldEncoder, TagMeasurer, TagReader, TagWriter,
    ValueEncoder, WireType, Wiretyped,
};
use crate::{Blob, DecodeError, DistinguishedMessage, Message};

use bytes::{Buf, BufMut, Bytes};

pub struct General;

// General implements unpacked encodings by default, but only for select collection types. Other
// implementers of the `Collection` trait must use Unpacked or Packed.
delegate_encoding!(delegate from General, to crate::encoding::Unpacked<General>,
    for type Vec<T>, including distinguished, with generics, T);
delegate_encoding!(delegate from General, to crate::encoding::Unpacked<General>,
    for type BTreeSet<T>, including distinguished, with generics, T);
delegate_encoding!(delegate from General, to crate::encoding::Map<General, General>,
    for type BTreeMap<K, V>, including distinguished, with generics, K, V);
#[cfg(feature = "std")]
delegate_encoding!(delegate from General, to crate::encoding::Unpacked<General>,
    for type HashSet<T>, with generics, T);
#[cfg(feature = "std")]
delegate_encoding!(delegate from General, to crate::encoding::Map<General, General>,
    for type HashMap<K, V>, including distinguished, with generics, K, V);

/// General encodes plain values only when they are non-default.
impl<T> Encoder<T> for General
where
    General: ValueEncoder<T>,
    T: Default + PartialEq,
{
    #[inline]
    fn encode<B: BufMut>(tag: u32, value: &T, buf: &mut B, tw: &mut TagWriter) {
        if *value != T::default() {
            Self::encode_field(tag, value, buf, tw);
        }
    }

    #[inline]
    fn encoded_len(tag: u32, value: &T, tm: &mut TagMeasurer) -> usize {
        if *value != T::default() {
            Self::field_encoded_len(tag, value, tm)
        } else {
            0
        }
    }

    #[inline]
    fn decode<B: Buf>(
        wire_type: WireType,
        duplicated: bool,
        value: &mut T,
        buf: &mut Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        if duplicated {
            return Err(DecodeError::new(
                "multiple occurrences of non-repeated field",
            ));
        }
        Self::decode_field(wire_type, value, buf, ctx)
    }
}

/// General's distinguished encoding for plain values forbids encoding defaulted values. This
/// includes directly-nested message types, which are not emitted when all their fields are default.
impl<T> DistinguishedEncoder<T> for General
where
    General: DistinguishedValueEncoder<T> + Encoder<T>,
    T: Default + Eq,
{
    #[inline]
    fn decode_distinguished<B: Buf>(
        wire_type: WireType,
        duplicated: bool,
        value: &mut T,
        buf: &mut Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        if duplicated {
            return Err(DecodeError::new(
                "multiple occurrences of non-repeated field",
            ));
        }
        Self::decode_field_distinguished(wire_type, value, buf, ctx)?;
        if *value == T::default() {
            return Err(DecodeError::new(
                "plain field was encoded with its zero value",
            ));
        }
        Ok(())
    }
}

/// Macro which emits implementations for variable width numeric encoding.
macro_rules! varint {
    (
        $name:ident,
        $ty:ty,
        to_uint64($to_uint64_value:ident) $to_uint64:expr,
        from_uint64($from_uint64_value:ident) $from_uint64:expr
    ) => {
        impl Wiretyped<$ty> for General {
            const WIRE_TYPE: WireType = WireType::Varint;
        }

        impl ValueEncoder<$ty> for General {
            #[inline]
            fn encode_value<B: BufMut>($to_uint64_value: &$ty, buf: &mut B) {
                encode_varint($to_uint64, buf);
            }

            #[inline]
            fn value_encoded_len($to_uint64_value: &$ty) -> usize {
                encoded_len_varint($to_uint64)
            }

            #[inline]
            fn decode_value<B: Buf>(
                __value: &mut $ty,
                buf: &mut Capped<B>,
                _ctx: DecodeContext,
            ) -> Result<(), DecodeError> {
                let $from_uint64_value = buf.decode_varint()?;
                *__value = $from_uint64;
                Ok(())
            }
        }

        impl DistinguishedValueEncoder<$ty> for General {
            #[inline]
            fn decode_value_distinguished<B: Buf>(
                value: &mut $ty,
                buf: &mut Capped<B>,
                ctx: DecodeContext,
            ) -> Result<(), DecodeError> {
                Self::decode_value(value, buf, ctx)
            }
        }

        #[cfg(test)]
        mod $name {
            crate::encoding::check_type_test!(General, expedient, $ty, WireType::Varint);
            crate::encoding::check_type_test!(General, distinguished, $ty, WireType::Varint);
        }
    };
}

varint!(varint_bool, bool,
to_uint64(value) {
    u64::from(*value)
},
from_uint64(value) {
    match value {
        0 => false,
        1 => true,
        _ => return Err(DecodeError::new("invalid varint value for bool"))
    }
});
varint!(varint_u32, u32,
to_uint64(value) {
    *value as u64
},
from_uint64(value) {
    u32::try_from(value).map_err(|_| DecodeError::new("varint overflows range of u32"))?
});
varint!(varint_u64, u64,
to_uint64(value) {
    *value
},
from_uint64(value) {
    value
});
varint!(varint_i32, i32,
to_uint64(value) {
    ((value << 1) ^ (value >> 31)) as u32 as u64
},
from_uint64(value) {
    let value = u32::try_from(value)
        .map_err(|_| DecodeError::new("varint overflows range of i32"))?;
    ((value >> 1) as i32) ^ (-((value & 1) as i32))
});
varint!(varint_i64, i64,
to_uint64(value) {
    ((value << 1) ^ (value >> 63)) as u64
},
from_uint64(value) {
    ((value >> 1) as i64) ^ (-((value & 1) as i64))
});

// General also encodes floating point values.
delegate_value_encoding!(delegate from General, to crate::encoding::Fixed, for type f32);
delegate_value_encoding!(delegate from General, to crate::encoding::Fixed, for type f64);

impl Wiretyped<String> for General {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}

// TODO(widders): rope string? Cow string? cow string is probably pretty doable. does it matter?

impl ValueEncoder<String> for General {
    fn encode_value<B: BufMut>(value: &String, buf: &mut B) {
        encode_varint(value.len() as u64, buf);
        buf.put_slice(value.as_bytes());
    }

    fn value_encoded_len(value: &String) -> usize {
        encoded_len_varint(value.len() as u64) + value.len()
    }

    fn decode_value<B: Buf>(
        value: &mut String,
        buf: &mut Capped<B>,
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
        unsafe {
            struct DropGuard<'a>(&'a mut Vec<u8>);
            impl<'a> Drop for DropGuard<'a> {
                #[inline]
                fn drop(&mut self) {
                    self.0.clear();
                }
            }

            let source = buf.take_length_delimited()?.take_all();
            // If we must copy, make sure to copy only once.
            value.clear();
            value.reserve(source.remaining());
            let drop_guard = DropGuard(value.as_mut_vec());
            drop_guard.0.put(source);
            match str::from_utf8(drop_guard.0) {
                Ok(_) => {
                    // Success; do not clear the bytes.
                    mem::forget(drop_guard);
                    Ok(())
                }
                Err(_) => Err(DecodeError::new(
                    "invalid string value: data is not UTF-8 encoded",
                )),
            }
        }
    }
}

impl DistinguishedValueEncoder<String> for General {
    fn decode_value_distinguished<B: Buf>(
        value: &mut String,
        buf: &mut Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        Self::decode_value(value, buf, ctx)
    }
}

#[cfg(test)]
mod string {
    use crate::encoding::check_type_test;
    check_type_test!(
        General,
        expedient,
        alloc::string::String,
        WireType::LengthDelimited
    );
    check_type_test!(
        General,
        distinguished,
        alloc::string::String,
        WireType::LengthDelimited
    );
}

impl Wiretyped<Bytes> for General {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}

impl ValueEncoder<Bytes> for General {
    fn encode_value<B: BufMut>(value: &Bytes, buf: &mut B) {
        encode_varint(value.len() as u64, buf);
        buf.put(value.clone());
    }

    fn value_encoded_len(value: &Bytes) -> usize {
        encoded_len_varint(value.len() as u64) + value.len()
    }

    fn decode_value<B: Buf>(
        value: &mut Bytes,
        buf: &mut Capped<B>,
        _ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        let mut buf = buf.take_length_delimited()?;
        let len = buf.remaining_before_cap();
        *value = buf.copy_to_bytes(len);
        Ok(())
    }
}

impl DistinguishedValueEncoder<Bytes> for General {
    fn decode_value_distinguished<B: Buf>(
        value: &mut Bytes,
        buf: &mut Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        Self::decode_value(value, buf, ctx)
    }
}

#[cfg(test)]
mod bytes_blob {
    use crate::encoding::check_type_test;
    check_type_test!(
        General,
        expedient,
        from Vec<u8>,
        into bytes::Bytes,
        WireType::LengthDelimited
    );
    check_type_test!(
        General,
        distinguished,
        from Vec<u8>,
        into bytes::Bytes,
        WireType::LengthDelimited
    );
}

impl Wiretyped<Blob> for General {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}

impl ValueEncoder<Blob> for General {
    #[inline]
    fn encode_value<B: BufMut>(value: &Blob, buf: &mut B) {
        crate::encoding::VecBlob::encode_value(value, buf)
    }

    #[inline]
    fn value_encoded_len(value: &Blob) -> usize {
        crate::encoding::VecBlob::value_encoded_len(value)
    }

    #[inline]
    fn decode_value<B: Buf>(
        value: &mut Blob,
        buf: &mut Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        crate::encoding::VecBlob::decode_value(value, buf, ctx)
    }
}

impl DistinguishedValueEncoder<Blob> for General {
    #[inline]
    fn decode_value_distinguished<B: Buf>(
        value: &mut Blob,
        buf: &mut Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        crate::encoding::VecBlob::decode_value_distinguished(value, buf, ctx)
    }
}

#[cfg(test)]
mod blob {
    use crate::encoding::check_type_test;
    check_type_test!(General, expedient, crate::Blob, WireType::LengthDelimited);
    check_type_test!(
        General,
        distinguished,
        crate::Blob,
        WireType::LengthDelimited
    );
}

// TODO(widders): Oneof

impl<T> Wiretyped<T> for General
where
    T: Message,
{
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}

impl<T> ValueEncoder<T> for General
where
    T: Message,
{
    fn encode_value<B: BufMut>(value: &T, buf: &mut B) {
        // TODO(widders): care needs to be taken with top level APIs to avoid running over when
        //  encoding and panicking in the buf
        encode_varint(value.encoded_len() as u64, buf);
        value.encode_raw(buf);
    }

    fn value_encoded_len(value: &T) -> usize {
        let inner_len = value.encoded_len();
        encoded_len_varint(inner_len as u64) + inner_len
    }

    fn decode_value<B: Buf>(
        value: &mut T,
        buf: &mut Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        // TODO(widders): this will get cleaned up
        message::merge(WireType::LengthDelimited, value, buf, ctx)
    }
}

impl<T> DistinguishedValueEncoder<T> for General
where
    T: DistinguishedMessage,
{
    fn decode_value_distinguished<B: Buf>(
        _value: &mut T,
        _buf: &mut Capped<B>,
        _ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        todo!()
    }
}

// TODO(widders): delete this module
pub mod message {
    use super::*;

    pub fn encode<M, B>(tag: u32, msg: &M, buf: &mut B, tw: &mut TagWriter)
    where
        M: Message,
        B: BufMut,
    {
        tw.encode_key(tag, WireType::LengthDelimited, buf);
        encode_varint(msg.encoded_len() as u64, buf);
        msg.encode_raw(buf);
    }

    pub fn merge<M, B>(
        wire_type: WireType,
        msg: &mut M,
        buf: &mut Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError>
    where
        M: Message,
        B: Buf,
    {
        check_wire_type(WireType::LengthDelimited, wire_type)?;
        ctx.limit_reached()?;
        let mut tr = TagReader::new();
        let inner_ctx = ctx.enter_recursion();
        let mut last_tag = None::<u32>;
        buf.take_length_delimited()?
            .consume(|buf| {
                let (tag, wire_type) = tr.decode_key(buf.buf())?;
                let duplicated = last_tag == Some(tag);
                last_tag = Some(tag);
                msg.merge_field(tag, wire_type, duplicated, buf, inner_ctx.clone())
            })
            .collect()
    }

    pub fn encode_repeated<M, B>(tag: u32, messages: &[M], buf: &mut B, tw: &mut TagWriter)
    where
        M: Message,
        B: BufMut,
    {
        for msg in messages {
            encode(tag, msg, buf, tw);
        }
    }

    pub fn merge_repeated<M, B>(
        wire_type: WireType,
        messages: &mut Vec<M>,
        buf: &mut Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError>
    where
        M: Message + Default,
        B: Buf,
    {
        check_wire_type(WireType::LengthDelimited, wire_type)?;
        let mut msg = M::default();
        merge(WireType::LengthDelimited, &mut msg, buf, ctx)?;
        messages.push(msg);
        Ok(())
    }

    #[inline]
    pub fn encoded_len<M: Message>(tag: u32, msg: &M, tm: &mut TagMeasurer) -> usize {
        let len = msg.encoded_len();
        tm.key_len(tag) + encoded_len_varint(len as u64) + len
    }

    #[inline]
    pub fn encoded_len_repeated<M: Message>(
        tag: u32,
        messages: &[M],
        tm: &mut TagMeasurer,
    ) -> usize {
        if messages.is_empty() {
            0
        } else {
            // successive repeated keys always take up 1 byte
            tm.key_len(tag) + messages.len() - 1
                + messages
                    .iter()
                    .map(Message::encoded_len)
                    .map(|len| len + encoded_len_varint(len as u64))
                    .sum::<usize>()
        }
    }
}
