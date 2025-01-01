use crate::buf::ReverseBuf;
use crate::encoding::{Capped, DecodeContext, DistinguishedValueEncoder, RestrictedDecodeContext, ValueEncoder, WireType, Wiretyped};
use crate::{Canonicity, DecodeError, DecodeErrorKind};
use bytes::{Buf, BufMut};
use core::ops::Deref;

/// Proxied is a special encoder which translates the encoded type into its "proxy" type first,
/// simplifying the encoding logic.
// TODO(widders): if this is published, consider adding a tag type to the parameters so proxy impls
//  can be sealed by their implementers
pub struct Proxied<E>(E);

pub(crate) trait Proxiable {
    type Proxy;

    fn new_proxy() -> Self::Proxy;

    fn encode_proxy(&self) -> Self::Proxy;

    fn decode_proxy(&mut self, proxy: Self::Proxy) -> Result<(), DecodeErrorKind>;
}

pub(crate) trait DistinguishedProxiable: Proxiable {
    fn decode_proxy_distinguished(
        &mut self,
        proxy: Self::Proxy,
    ) -> Result<Canonicity, DecodeErrorKind>;
}

impl<T, E> Wiretyped<Proxied<E>> for T
where
    T: Proxiable,
    T::Proxy: Wiretyped<E>,
{
    const WIRE_TYPE: WireType = T::Proxy::WIRE_TYPE;
}

impl<T, E> ValueEncoder<Proxied<E>> for T
where
    T: Proxiable,
    T::Proxy: ValueEncoder<E>,
{
    #[inline]
    fn encode_value<B: BufMut + ?Sized>(value: &Self, buf: &mut B) {
        ValueEncoder::<E>::encode_value(&value.encode_proxy(), buf);
    }

    #[inline]
    fn prepend_value<B: ReverseBuf + ?Sized>(value: &Self, buf: &mut B) {
        ValueEncoder::<E>::prepend_value(&value.encode_proxy(), buf);
    }

    #[inline]
    fn value_encoded_len(value: &Self) -> usize {
        ValueEncoder::<E>::value_encoded_len(&value.encode_proxy())
    }

    #[inline]
    fn many_values_encoded_len<I>(values: I) -> usize
    where
        I: ExactSizeIterator,
        I::Item: Deref<Target = Self>,
    {
        /// Do-nothing wrapper allowing us to return items by-value and still have them Deref to T. Maybe
        /// it would be "more correct" to use Borrow or something like that but this is pretty easy too.
        #[repr(transparent)]
        struct WrapDeref<T>(T);

        impl<T> Deref for WrapDeref<T> {
            type Target = T;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        ValueEncoder::<E>::many_values_encoded_len(
            values.map(|item| WrapDeref(item.encode_proxy())),
        )
    }

    #[inline]
    fn decode_value<B: Buf + ?Sized>(
        value: &mut Self,
        buf: Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        let mut proxy = T::new_proxy();
        ValueEncoder::<E>::decode_value(&mut proxy, buf, ctx)?;
        Ok(value.decode_proxy(proxy)?)
    }
}

impl<T, E> DistinguishedValueEncoder<Proxied<E>> for T
where
    T: DistinguishedProxiable + Eq,
    T::Proxy: DistinguishedValueEncoder<E>,
{
    const CHECKS_EMPTY: bool = T::Proxy::CHECKS_EMPTY;

    fn decode_value_distinguished<const ALLOW_EMPTY: bool>(
        value: &mut Self,
        buf: Capped<impl Buf + ?Sized>,
        ctx: RestrictedDecodeContext,
    ) -> Result<Canonicity, DecodeError> {
        let mut proxy = T::new_proxy();
        let mut canon = DistinguishedValueEncoder::<E>::decode_value_distinguished::<ALLOW_EMPTY>(
            &mut proxy, buf, ctx,
        )?;
        canon.update(value.decode_proxy_distinguished(proxy)?);
        Ok(canon)
    }
}
