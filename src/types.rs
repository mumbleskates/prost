use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut};

use bytes::{Buf, BufMut};

use crate::buf::ReverseBuf;
use crate::encoding::{skip_field, Canonicity, Capped, DecodeContext, WireType};
use crate::message::{RawDistinguishedMessage, RawMessage};
use crate::DecodeError;

/// Newtype wrapper to act as a simple "bytes data" type in Bilrost. It transparently wraps a
/// `Vec<u8>` and is fully supported by the `General` encoder.
///
/// To use `Vec<u8>` directly, use the `PlainBytes` encoder.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
#[repr(transparent)]
pub struct Blob(Vec<u8>);

impl Blob {
    pub fn new() -> Self {
        Self::from_vec(Vec::new())
    }

    pub fn from_vec(vec: Vec<u8>) -> Self {
        Self(vec)
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

impl Deref for Blob {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Blob {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<Vec<u8>> for Blob {
    fn as_ref(&self) -> &Vec<u8> {
        &self.0
    }
}

impl AsMut<Vec<u8>> for Blob {
    fn as_mut(&mut self) -> &mut Vec<u8> {
        &mut self.0
    }
}

impl Borrow<Vec<u8>> for Blob {
    fn borrow(&self) -> &Vec<u8> {
        &self.0
    }
}

impl BorrowMut<Vec<u8>> for Blob {
    fn borrow_mut(&mut self) -> &mut Vec<u8> {
        &mut self.0
    }
}

impl From<Vec<u8>> for Blob {
    fn from(value: Vec<u8>) -> Self {
        Blob::from_vec(value)
    }
}

impl From<Blob> for Vec<u8> {
    fn from(value: Blob) -> Self {
        value.0
    }
}

impl From<&[u8]> for Blob {
    fn from(value: &[u8]) -> Self {
        Self(value.into())
    }
}

impl From<&mut [u8]> for Blob {
    fn from(value: &mut [u8]) -> Self {
        Self(value.into())
    }
}

impl<const N: usize> From<&[u8; N]> for Blob {
    fn from(value: &[u8; N]) -> Self {
        // MSRV: as_slice() needed
        Self(value.as_slice().into())
    }
}

impl<const N: usize> From<[u8; N]> for Blob {
    fn from(value: [u8; N]) -> Self {
        Self(value.into())
    }
}

impl From<Cow<'_, [u8]>> for Blob {
    fn from(value: Cow<[u8]>) -> Self {
        Self(value.into())
    }
}

impl From<Box<[u8]>> for Blob {
    fn from(value: Box<[u8]>) -> Self {
        Self(value.into())
    }
}

impl From<&str> for Blob {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

#[cfg(test)]
impl proptest::arbitrary::Arbitrary for Blob {
    type Parameters = <Vec<u8> as proptest::arbitrary::Arbitrary>::Parameters;
    fn arbitrary_with(top: Self::Parameters) -> Self::Strategy {
        proptest::strategy::Strategy::prop_map(
            proptest::arbitrary::any_with::<Vec<u8>>(top),
            Blob::from_vec,
        )
    }
    type Strategy = proptest::strategy::Map<
        <Vec<u8> as proptest::arbitrary::Arbitrary>::Strategy,
        fn(Vec<u8>) -> Self,
    >;
}

/// The empty tuple unit is the only native tuple type that implements Message because there are no
/// choices to be made about how its fields will be encoded. All other native tuples are only
/// implemented as field values. They encode exactly as if they were nested messages, but their
/// encoding must be specified.
impl RawMessage for () {
    const __ASSERTIONS: () = ();

    fn raw_encode<B: BufMut + ?Sized>(&self, _buf: &mut B) {}

    fn raw_prepend<B: ReverseBuf + ?Sized>(&self, _buf: &mut B) {}

    fn raw_encoded_len(&self) -> usize {
        0
    }

    fn raw_decode_field<B: Buf + ?Sized>(
        &mut self,
        _tag: u32,
        wire_type: WireType,
        _duplicated: bool,
        buf: Capped<B>,
        _ctx: DecodeContext,
    ) -> Result<(), DecodeError>
    where
        Self: Sized,
    {
        skip_field(wire_type, buf)
    }
}

impl RawDistinguishedMessage for () {
    fn raw_decode_field_distinguished<B: Buf + ?Sized>(
        &mut self,
        _tag: u32,
        wire_type: WireType,
        _duplicated: bool,
        buf: Capped<B>,
        _ctx: DecodeContext,
    ) -> Result<Canonicity, DecodeError>
    where
        Self: Sized,
    {
        skip_field(wire_type, buf)?;
        Ok(Canonicity::HasExtensions)
    }
}
