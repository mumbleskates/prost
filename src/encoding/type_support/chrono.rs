use crate::buf::ReverseBuf;
use crate::encoding::local_proxy::LocalProxy;
use crate::encoding::underived::{
    underived_decode, underived_decode_distinguished, underived_encode, underived_encoded_len,
    underived_prepend,
};
use crate::encoding::value_traits::empty_state_via_default;
use crate::encoding::{
    delegate_value_encoding, Canonicity, Capped, DecodeContext, DecodeErrorKind,
    DistinguishedEncoder, DistinguishedProxiable, DistinguishedValueEncoder, EmptyState, Encoder,
    Fixed, ForOverwrite, General, Packed, Proxiable, Proxied, ValueEncoder, Varint, WireType,
    Wiretyped,
};
use crate::Canonicity::Canonical;
use crate::DecodeError;
use crate::DecodeErrorKind::{InvalidValue, OutOfDomainValue};
use bytes::{Buf, BufMut};
use chrono::{
    DateTime, Datelike, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, TimeZone,
    Timelike, Utc,
};

impl ForOverwrite for NaiveDate {
    fn for_overwrite() -> Self {
        Self::from_yo_opt(0, 1).unwrap()
    }
}

impl EmptyState for NaiveDate {
    fn is_empty(&self) -> bool {
        (self.year(), self.ordinal0()) == (0, 0)
    }

    fn clear(&mut self) {
        *self = Self::empty();
    }
}

impl Proxiable for NaiveDate {
    type Proxy = LocalProxy<i32, 2>;

    fn new_proxy() -> Self::Proxy {
        Self::Proxy::new_empty()
    }

    fn encode_proxy(&self) -> Self::Proxy {
        Self::Proxy::new_without_empty_suffix([self.year(), self.ordinal0() as i32])
    }

    fn decode_proxy(&mut self, proxy: Self::Proxy) -> Result<(), DecodeErrorKind> {
        let [year, ordinal0] = proxy.into_inner();
        let ordinal0: u32 = ordinal0.try_into().map_err(|_| OutOfDomainValue)?;
        *self = NaiveDate::from_yo_opt(year, ordinal0 + 1).ok_or(OutOfDomainValue)?;
        Ok(())
    }
}

impl DistinguishedProxiable for NaiveDate {
    fn decode_proxy_distinguished(
        &mut self,
        proxy: Self::Proxy,
    ) -> Result<Canonicity, DecodeErrorKind> {
        let ([year, ordinal0], canon) = proxy.into_inner_distinguished();
        let ordinal0: u32 = ordinal0.try_into().map_err(|_| OutOfDomainValue)?;
        *self = NaiveDate::from_yo_opt(year, ordinal0 + 1).ok_or(OutOfDomainValue)?;
        Ok(canon)
    }
}

// NaiveDate encodes as a packed sequence of signed varints with trailing zeros cut off:
// [year, ordinal day in year (starting at zero)]. The empty value is January 1st on the year 0,
// not 1970.
delegate_value_encoding!(delegate from (General) to (Proxied<Packed<Varint>>)
    for type (NaiveDate) including distinguished);

#[cfg(test)]
mod naivedate {
    use crate::encoding::test::{check_type_empty, check_type_test};
    use crate::encoding::General;
    use alloc::vec::Vec;
    use chrono::NaiveDate;

    check_type_empty!(NaiveDate, via proxy);
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
    check_type_empty!(NaiveDate, via distinguished proxy);
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

impl ForOverwrite for NaiveTime {
    fn for_overwrite() -> Self {
        Self::from_num_seconds_from_midnight_opt(0, 0).unwrap()
    }
}

impl EmptyState for NaiveTime {
    fn is_empty(&self) -> bool {
        (self.num_seconds_from_midnight(), self.nanosecond()) == (0, 0)
    }

    fn clear(&mut self) {
        *self = Self::empty();
    }
}

impl Proxiable for NaiveTime {
    type Proxy = LocalProxy<u32, 4>;

    fn new_proxy() -> Self::Proxy {
        Self::Proxy::new_empty()
    }

    fn encode_proxy(&self) -> Self::Proxy {
        Self::Proxy::new_without_empty_suffix([
            self.hour(),
            self.minute(),
            self.second(),
            self.nanosecond(),
        ])
    }

    fn decode_proxy(&mut self, proxy: Self::Proxy) -> Result<(), DecodeErrorKind> {
        let [hour, min, sec, nano] = proxy.into_inner();
        *self = Self::from_hms_nano_opt(hour, min, sec, nano).ok_or(OutOfDomainValue)?;
        Ok(())
    }
}

impl DistinguishedProxiable for NaiveTime {
    fn decode_proxy_distinguished(
        &mut self,
        proxy: Self::Proxy,
    ) -> Result<Canonicity, DecodeErrorKind> {
        let ([hour, min, sec, nano], canon) = proxy.into_inner_distinguished();
        *self = Self::from_hms_nano_opt(hour, min, sec, nano).ok_or(OutOfDomainValue)?;
        Ok(canon)
    }
}

// NaiveTime encodes as a packed sequence of UNsigned varints with trailing zeros cut off:
// [hour, minute, second, nanosecond].
delegate_value_encoding!(delegate from (General) to (Proxied<Packed<Varint>>)
    for type (NaiveTime) including distinguished);

#[cfg(test)]
mod naivetime {
    use crate::encoding::test::{check_type_empty, check_type_test};
    use crate::encoding::General;
    use alloc::vec::Vec;
    use chrono::NaiveTime;

    check_type_empty!(NaiveTime, via proxy);
    check_type_test!(
        General,
        expedient,
        from Vec<u8>,
        into NaiveTime,
        converter(b) {
            use arbitrary::{Arbitrary, Unstructured};
            NaiveTime::arbitrary(&mut Unstructured::new(&b)).unwrap()
        },
        WireType::LengthDelimited
    );
    check_type_empty!(NaiveTime, via distinguished proxy);
    check_type_test!(
        General,
        distinguished,
        from Vec<u8>,
        into NaiveTime,
        converter(b) {
            use arbitrary::{Arbitrary, Unstructured};
            NaiveTime::arbitrary(&mut Unstructured::new(&b)).unwrap()
        },
        WireType::LengthDelimited
    );
}

impl ForOverwrite for NaiveDateTime {
    fn for_overwrite() -> Self {
        Self::new(EmptyState::empty(), EmptyState::empty())
    }
}

impl EmptyState for NaiveDateTime {
    fn is_empty(&self) -> bool {
        (
            self.year(),
            self.ordinal0(),
            self.num_seconds_from_midnight(),
            self.nanosecond(),
        ) == (0, 0, 0, 0)
    }

    fn clear(&mut self) {
        *self = Self::empty();
    }
}

impl Proxiable for NaiveDateTime {
    type Proxy = LocalProxy<i32, 6>;

    fn new_proxy() -> Self::Proxy {
        Self::Proxy::new_empty()
    }

    fn encode_proxy(&self) -> Self::Proxy {
        Self::Proxy::new_without_empty_suffix([
            self.year(),
            self.ordinal0() as i32,
            self.hour() as i32,
            self.minute() as i32,
            self.second() as i32,
            self.nanosecond() as i32,
        ])
    }

    fn decode_proxy(&mut self, proxy: Self::Proxy) -> Result<(), DecodeErrorKind> {
        let [year, ordinal0, hour, min, sec, nano] = proxy.into_inner();
        let [ordinal0, hour, min, sec, nano]: [u32; 5] = [
            ordinal0.try_into().map_err(|_| OutOfDomainValue)?,
            hour.try_into().map_err(|_| OutOfDomainValue)?,
            min.try_into().map_err(|_| OutOfDomainValue)?,
            sec.try_into().map_err(|_| OutOfDomainValue)?,
            nano.try_into().map_err(|_| OutOfDomainValue)?,
        ];
        *self = Self::new(
            NaiveDate::from_yo_opt(year, ordinal0 + 1).ok_or(OutOfDomainValue)?,
            NaiveTime::from_hms_nano_opt(hour, min, sec, nano).ok_or(OutOfDomainValue)?,
        );
        Ok(())
    }
}

impl DistinguishedProxiable for NaiveDateTime {
    fn decode_proxy_distinguished(
        &mut self,
        proxy: Self::Proxy,
    ) -> Result<Canonicity, DecodeErrorKind> {
        let ([year, ordinal0, hour, min, sec, nano], canon) = proxy.into_inner_distinguished();
        let [ordinal0, hour, min, sec, nano]: [u32; 5] = [
            ordinal0.try_into().map_err(|_| OutOfDomainValue)?,
            hour.try_into().map_err(|_| OutOfDomainValue)?,
            min.try_into().map_err(|_| OutOfDomainValue)?,
            sec.try_into().map_err(|_| OutOfDomainValue)?,
            nano.try_into().map_err(|_| OutOfDomainValue)?,
        ];
        *self = Self::new(
            NaiveDate::from_yo_opt(year, ordinal0 + 1).ok_or(OutOfDomainValue)?,
            NaiveTime::from_hms_nano_opt(hour, min, sec, nano).ok_or(OutOfDomainValue)?,
        );
        Ok(canon)
    }
}

// NaiveDateTime encodes as a packed sequence of signed varints with trailing zeros cut off:
// [year, ordinal day in year (starting at zero), hour, minute, second, nanosecond]. It can decode
// NaiveDate values as if they were truncated NaiveDateTimes. The empty value is midnight on January
// 1st of the year 0, not 1970.
delegate_value_encoding!(delegate from (General) to (Proxied<Packed<Varint>>)
    for type (NaiveDateTime) including distinguished);

#[cfg(test)]
mod naivedatetime {
    use crate::encoding::test::{check_type_empty, check_type_test};
    use crate::encoding::General;
    use alloc::vec::Vec;
    use chrono::NaiveDateTime;

    check_type_empty!(NaiveDateTime, via proxy);
    check_type_test!(
        General,
        expedient,
        from Vec<u8>,
        into NaiveDateTime,
        converter(b) {
            use arbitrary::{Arbitrary, Unstructured};
            NaiveDateTime::arbitrary(&mut Unstructured::new(&b)).unwrap()
        },
        WireType::LengthDelimited
    );
    check_type_empty!(NaiveDateTime, via distinguished proxy);
    check_type_test!(
        General,
        distinguished,
        from Vec<u8>,
        into NaiveDateTime,
        converter(b) {
            use arbitrary::{Arbitrary, Unstructured};
            NaiveDateTime::arbitrary(&mut Unstructured::new(&b)).unwrap()
        },
        WireType::LengthDelimited
    );
}

impl ForOverwrite for Utc {
    fn for_overwrite() -> Self {
        Self
    }
}

impl EmptyState for Utc {
    fn is_empty(&self) -> bool {
        true
    }

    fn clear(&mut self) {}
}

impl Proxiable for Utc {
    type Proxy = (i8, i8, i8);

    fn new_proxy() -> Self::Proxy {
        (0, 0, 0)
    }

    fn encode_proxy(&self) -> Self::Proxy {
        Self::new_proxy()
    }

    fn decode_proxy(&mut self, proxy: Self::Proxy) -> Result<(), DecodeErrorKind> {
        if proxy == Self::new_proxy() {
            Ok(())
        } else {
            Err(OutOfDomainValue)
        }
    }
}

impl DistinguishedProxiable for Utc {
    fn decode_proxy_distinguished(
        &mut self,
        proxy: Self::Proxy,
    ) -> Result<Canonicity, DecodeErrorKind> {
        self.decode_proxy(proxy)?;
        Ok(Canonical)
    }
}

// The encoding for Utc is the same as the encoding for FixedOffset: it's a tuple of three signed
// varints (hour, minute, second) which are always zero. It always fails to decode when they are not
// all zero.
delegate_value_encoding!(delegate from (General) to (Proxied<(Varint, Varint, Varint)>)
    for type (Utc) including distinguished);

#[cfg(test)]
mod utc {
    use crate::encoding::Capped;
    use crate::encoding::DecodeContext;
    use crate::encoding::General;
    use crate::encoding::{DistinguishedValueEncoder, ForOverwrite, ValueEncoder};
    use crate::Canonicity::Canonical;
    use crate::DecodeError;
    use crate::DecodeErrorKind::OutOfDomainValue;
    use alloc::vec::Vec;
    use chrono::{FixedOffset, Utc};

    #[test]
    fn utc_rejects_nonzero_offsets() {
        {
            let mut buf = Vec::new();
            let zero_offset = FixedOffset::east_opt(0).unwrap();
            ValueEncoder::<General>::encode_value(&zero_offset, &mut buf);
            let mut utc = Utc::for_overwrite();
            assert_eq!(
                ValueEncoder::<General>::decode_value(
                    &mut utc,
                    Capped::new(&mut buf.as_slice()),
                    DecodeContext::default()
                ),
                Ok(())
            );
            assert_eq!(
                DistinguishedValueEncoder::<General>::decode_value_distinguished::<true>(
                    &mut utc,
                    Capped::new(&mut buf.as_slice()),
                    DecodeContext::default()
                ),
                Ok(Canonical)
            );
        }

        {
            let mut buf = Vec::new();
            let nonzero_offset = FixedOffset::east_opt(1000).unwrap();
            ValueEncoder::<General>::encode_value(&nonzero_offset, &mut buf);
            let mut utc = Utc::for_overwrite();
            assert_eq!(
                ValueEncoder::<General>::decode_value(
                    &mut utc,
                    Capped::new(&mut buf.as_slice()),
                    DecodeContext::default()
                ),
                Err(DecodeError::new(OutOfDomainValue))
            );
            assert_eq!(
                DistinguishedValueEncoder::<General>::decode_value_distinguished::<true>(
                    &mut utc,
                    Capped::new(&mut buf.as_slice()),
                    DecodeContext::default()
                ),
                Err(DecodeError::new(OutOfDomainValue))
            );
        }
    }
}

impl ForOverwrite for FixedOffset {
    fn for_overwrite() -> Self {
        FixedOffset::east_opt(0).unwrap()
    }
}

impl EmptyState for FixedOffset {
    fn is_empty(&self) -> bool {
        self.local_minus_utc() == 0
    }

    fn clear(&mut self) {
        *self = Self::empty();
    }
}

impl Proxiable for FixedOffset {
    type Proxy = (i8, i8, i8);

    fn new_proxy() -> Self::Proxy {
        (0, 0, 0)
    }

    fn encode_proxy(&self) -> Self::Proxy {
        let offset_secs = self.local_minus_utc();
        let secs = (offset_secs % 60) as i8;
        let offset_mins = offset_secs / 60;
        let mins = (offset_mins % 60) as i8;
        let hours = (offset_mins / 60) as i8;
        (hours, mins, secs)
    }

    fn decode_proxy(&mut self, proxy: Self::Proxy) -> Result<(), DecodeErrorKind> {
        let offset_secs = match proxy {
            (hours @ -23..=23, mins @ -59..=59, secs @ -59..=59) => {
                let total_offset = (hours as i32) * 60 * 60 + (mins as i32) * 60 + (secs as i32);

                // offsets should always have the same sign for all three components; we don't want
                // any two offsets to have the same total via different combinations.
                //
                // we enforce this even in expedient mode because dealing with time is already bad
                // enough.
                let mut signums = [false; 3];
                for component in [hours, mins, secs] {
                    signums[(component.signum() + 1) as usize] = true;
                }
                if let [true, _, true] = signums {
                    return Err(InvalidValue);
                }

                total_offset
            }
            _ => return Err(OutOfDomainValue),
        };
        *self = Self::east_opt(offset_secs).unwrap();
        Ok(())
    }
}

impl DistinguishedProxiable for FixedOffset {
    fn decode_proxy_distinguished(
        &mut self,
        proxy: Self::Proxy,
    ) -> Result<Canonicity, DecodeErrorKind> {
        self.decode_proxy(proxy)?;
        Ok(Canonical)
    }
}

// The encoding for FixedOffset is (hour, minute, second) as a basic tuple of signed varints. It
// It fails to decode whenever the components have mixed signs or are out of range.
delegate_value_encoding!(delegate from (General) to (Proxied<(Varint, Varint, Varint)>)
    for type (FixedOffset) including distinguished);

#[cfg(test)]
mod fixedoffset {
    use crate::encoding::test::{check_type_empty, check_type_test};
    use crate::encoding::value_traits::ForOverwrite;
    use crate::encoding::{
        Capped, DecodeContext, DistinguishedValueEncoder, General, ValueEncoder,
    };
    use crate::DecodeError;
    use crate::DecodeErrorKind::{InvalidValue, OutOfDomainValue};
    use alloc::vec::Vec;
    use chrono::FixedOffset;

    check_type_empty!(FixedOffset, via proxy);
    check_type_test!(
        General,
        expedient,
        from Vec<u8>,
        into FixedOffset,
        converter(b) {
            use arbitrary::{Arbitrary, Unstructured};
            FixedOffset::arbitrary(&mut Unstructured::new(&b)).unwrap()
        },
        WireType::LengthDelimited
    );
    check_type_empty!(FixedOffset, via distinguished proxy);
    check_type_test!(
        General,
        distinguished,
        from Vec<u8>,
        into FixedOffset,
        converter(b) {
            use arbitrary::{Arbitrary, Unstructured};
            FixedOffset::arbitrary(&mut Unstructured::new(&b)).unwrap()
        },
        WireType::LengthDelimited
    );

    #[test]
    fn fixedoffset_rejects_out_of_range() {
        {
            let mut buf = Vec::new();
            let out_of_range: (i32, i32, i32) = (23, 45, 67);
            ValueEncoder::<General>::encode_value(&out_of_range, &mut buf);
            let mut fixed = FixedOffset::for_overwrite();
            assert_eq!(
                ValueEncoder::<General>::decode_value(
                    &mut fixed,
                    Capped::new(&mut buf.as_slice()),
                    DecodeContext::default()
                ),
                Err(DecodeError::new(OutOfDomainValue))
            );
            assert_eq!(
                DistinguishedValueEncoder::<General>::decode_value_distinguished::<true>(
                    &mut fixed,
                    Capped::new(&mut buf.as_slice()),
                    DecodeContext::default()
                ),
                Err(DecodeError::new(OutOfDomainValue))
            );
        }
    }

    #[test]
    fn fixedoffset_rejects_mixed_signs() {
        {
            let mut buf = Vec::new();
            let out_of_range: (i32, i32, i32) = (10, 0, -10);
            ValueEncoder::<General>::encode_value(&out_of_range, &mut buf);
            let mut fixed = FixedOffset::for_overwrite();
            assert_eq!(
                ValueEncoder::<General>::decode_value(
                    &mut fixed,
                    Capped::new(&mut buf.as_slice()),
                    DecodeContext::default()
                ),
                Err(DecodeError::new(InvalidValue))
            );
            assert_eq!(
                DistinguishedValueEncoder::<General>::decode_value_distinguished::<true>(
                    &mut fixed,
                    Capped::new(&mut buf.as_slice()),
                    DecodeContext::default()
                ),
                Err(DecodeError::new(InvalidValue))
            );
        }
    }
}

impl<Z> ForOverwrite for DateTime<Z>
where
    Z: TimeZone,
    Z::Offset: EmptyState,
{
    fn for_overwrite() -> Self {
        Self::from_naive_utc_and_offset(EmptyState::empty(), EmptyState::empty())
    }
}

impl<Z> EmptyState for DateTime<Z>
where
    Z: TimeZone,
    Z::Offset: EmptyState,
{
    fn is_empty(&self) -> bool {
        self.naive_utc().is_empty() && self.offset().is_empty()
    }

    fn clear(&mut self) {
        *self = Self::empty();
    }
}

impl<Z> Proxiable for DateTime<Z>
where
    Z: TimeZone,
    Z::Offset: EmptyState,
{
    type Proxy = (NaiveDateTime, Z::Offset);

    fn new_proxy() -> Self::Proxy {
        Self::Proxy::empty()
    }

    fn encode_proxy(&self) -> Self::Proxy {
        (self.naive_utc(), self.offset().clone())
    }

    fn decode_proxy(&mut self, proxy: Self::Proxy) -> Result<(), DecodeErrorKind> {
        let (naive, offset) = proxy;
        *self = Self::from_naive_utc_and_offset(naive, offset);
        Ok(())
    }
}

impl<Z> DistinguishedProxiable for DateTime<Z>
where
    Z: TimeZone,
    Z::Offset: EmptyState,
{
    fn decode_proxy_distinguished(
        &mut self,
        proxy: Self::Proxy,
    ) -> Result<Canonicity, DecodeErrorKind> {
        self.decode_proxy(proxy)?;
        Ok(Canonical)
    }
}

// The encoding for DateTime<Tz> is the same as the (NaiveDateTime, Tz::Offset) that it is composed
// of.
delegate_value_encoding!(delegate from (General) to (Proxied<General>)
    for type (DateTime<Z>) including distinguished
    with where clause for expedient (Z: TimeZone, Z::Offset: EmptyState)
    with generics (Z));

#[cfg(test)]
mod datetime {
    use crate::encoding::test::{check_type_empty, check_type_test};
    use crate::encoding::General;
    use alloc::vec::Vec;
    use chrono::{DateTime, Utc};

    check_type_empty!(DateTime<Utc>, via proxy);
    check_type_test!(
        General,
        expedient,
        from Vec<u8>,
        into DateTime<Utc>,
        converter(b) {
            use arbitrary::{Arbitrary, Unstructured};
            DateTime::<Utc>::arbitrary(&mut Unstructured::new(&b)).unwrap()
        },
        WireType::LengthDelimited
    );
    check_type_empty!(DateTime<Utc>, via distinguished proxy);
    check_type_test!(
        General,
        distinguished,
        from Vec<u8>,
        into DateTime<Utc>,
        converter(b) {
            use arbitrary::{Arbitrary, Unstructured};
            DateTime::<Utc>::arbitrary(&mut Unstructured::new(&b)).unwrap()
        },
        WireType::LengthDelimited
    );
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct TimeDeltaProxy {
    secs: i64,
    nanos: i32,
}

empty_state_via_default!(TimeDeltaProxy);

impl Wiretyped<General> for TimeDeltaProxy {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}

impl ValueEncoder<General> for TimeDeltaProxy {
    fn encode_value<B: BufMut + ?Sized>(value: &Self, buf: &mut B) {
        underived_encode!(TimeDelta {
            1: General => secs: &value.secs,
            2: Fixed => nanos: &value.nanos,
        }, buf)
    }

    fn prepend_value<B: ReverseBuf + ?Sized>(value: &Self, buf: &mut B) {
        underived_prepend!(TimeDelta {
            2: Fixed => nanos: &value.nanos,
            1: General => secs: &value.secs,
        }, buf)
    }

    fn value_encoded_len(value: &Self) -> usize {
        underived_encoded_len!(TimeDelta {
            1: General => secs: &value.secs,
            2: Fixed => nanos: &value.nanos,
        })
    }

    fn decode_value<B: Buf + ?Sized>(
        value: &mut Self,
        mut buf: Capped<B>,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        underived_decode!(TimeDelta {
            1: General => secs: &mut value.secs,
            2: Fixed => nanos: &mut value.nanos,
        }, buf, ctx)?;
        if value.secs.signum() as i32 * value.nanos.signum() == -1 {
            Err(DecodeError::new(InvalidValue))
        } else {
            Ok(())
        }
    }
}

impl DistinguishedValueEncoder<General> for TimeDeltaProxy {
    const CHECKS_EMPTY: bool = true;

    fn decode_value_distinguished<const ALLOW_EMPTY: bool>(
        value: &mut Self,
        mut buf: Capped<impl Buf + ?Sized>,
        ctx: DecodeContext,
    ) -> Result<Canonicity, DecodeError> {
        underived_decode_distinguished!(TimeDelta {
            1: General => secs: &mut value.secs,
            2: Fixed => nanos: &mut value.nanos,
        }, buf, ctx)
    }
}

empty_state_via_default!(TimeDelta);

impl Proxiable for TimeDelta {
    type Proxy = TimeDeltaProxy;

    fn new_proxy() -> Self::Proxy {
        TimeDeltaProxy::default()
    }

    fn encode_proxy(&self) -> Self::Proxy {
        TimeDeltaProxy {
            secs: self.num_seconds(),
            nanos: self.subsec_nanos(),
        }
    }

    fn decode_proxy(&mut self, proxy: Self::Proxy) -> Result<(), DecodeErrorKind> {
        const NEGATED_I64_MAX: i64 = -i64::MAX;

        let (secs, nanos) = match (proxy.secs, proxy.nanos) {
            // we must be able to subtract 1 from secs no matter what
            (secs @ NEGATED_I64_MAX..=0, nanos @ -999_999_999..=-1) => {
                (secs - 1, nanos + 1_000_000_000)
            }
            // we also ensure that the sign of secs and nanos matches and that nanos is in-bounds
            (secs @ 0.., nanos @ 0..=999_999_999) => (secs, nanos),
            _ => return Err(InvalidValue),
        };
        // TimeDelta only wants to be constructed from a u32 nanos, which is its internal repr, even
        // though it only gives the value back as an i32 with the same sign as the original.
        *self = Self::new(secs, nanos as u32).ok_or(OutOfDomainValue)?;
        Ok(())
    }
}

impl DistinguishedProxiable for TimeDelta {
    fn decode_proxy_distinguished(
        &mut self,
        proxy: Self::Proxy,
    ) -> Result<Canonicity, DecodeErrorKind> {
        self.decode_proxy(proxy)?;
        Ok(Canonical)
    }
}

// The encoding for TimeDelta matches that of bilrost_types::Duration.
delegate_value_encoding!(delegate from (General) to (Proxied<General>)
    for type (TimeDelta) including distinguished);

#[cfg(test)]
mod timedelta {
    use crate::encoding::test::check_type_empty;
    use crate::encoding::General;
    use chrono::TimeDelta;

    check_type_empty!(TimeDelta, via proxy);
    check_type_empty!(TimeDelta, via distinguished proxy);

    #[cfg(test)]
    mod check_type {
        use proptest::prelude::*;

        use super::*;
        use crate::encoding::WireType;

        fn milli_nanos_to_timedelta(millis: i64, submilli_nanos: u32, negative: bool) -> TimeDelta {
            // compute millisecond part
            let secs = millis / 1000;
            let nanos = (millis % 1000 * 1_000_000) as u32 + submilli_nanos;
            let td = TimeDelta::new(secs, nanos).unwrap();
            if negative {
                -td
            } else {
                td
            }
        }

        // we write these out because the arbitrary::Arbitrary impl for TimeDelta is, for some
        // reason, extremely fallible. The underlying data model for TimeDelta is also pretty weird,
        // in that its internal repr is (secs: i64, nanos: i32 /* always positive */), and it is
        // also documented to be restricted to a magnitude of plus or minus i64::MAX
        // *milliseconds* plus up to 999,999 nanoseconds, with a freely swappable sign.
        proptest! {
            #[test]
            fn check_expedient(
                millis in 0..i64::MAX,
                submilli_nanos in 0..=999_999u32,
                negative: bool,
                tag: u32,
            ) {
                crate::encoding::test::expedient::check_type::<TimeDelta, General>(
                    milli_nanos_to_timedelta(millis, submilli_nanos, negative),
                    tag,
                    WireType::LengthDelimited,
                )?;
            }
            #[test]
            fn check_distinguished(
                millis in 0..i64::MAX,
                submilli_nanos in 0..=999_999u32,
                negative: bool,
                tag: u32,
            ) {
                crate::encoding::test::distinguished::check_type::<TimeDelta, General>(
                    milli_nanos_to_timedelta(millis, submilli_nanos, negative),
                    tag,
                    WireType::LengthDelimited,
                )?;
            }
        }
    }
}
