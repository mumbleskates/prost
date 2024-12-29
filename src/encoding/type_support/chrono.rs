use crate::encoding::local_proxy::LocalProxy;
use crate::encoding::{
    delegate_value_encoding, DistinguishedProxiable, Packed, Proxiable, Proxied,
};
use crate::encoding::{Canonicity, DecodeErrorKind, EmptyState, ForOverwrite, General, Varint};
use crate::Canonicity::Canonical;
use crate::DecodeErrorKind::{InvalidValue, OutOfDomainValue};
use chrono::{
    DateTime, Datelike, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc,
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
            (hours @ -24..24, mins @ -60..60, secs @ -60..60) => {
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

// TODO(widders): finish these
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
//      * actually this is deprecated nvm
//  * struct DateTime<impl TimeZone>
//      * aggreagate of (NaiveDateTime, offset)
//      * store as tuple
//  * struct TimeDelta
//      * matches bilrost_types::Duration, but nanos is always positive
//      * use derived storage
