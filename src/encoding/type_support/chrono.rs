use crate::encoding::local_proxy::LocalProxy;
use crate::encoding::{
    delegate_value_encoding, DistinguishedProxiable, Packed, Proxiable, Proxied,
};
use crate::encoding::{Canonicity, DecodeErrorKind, EmptyState, ForOverwrite, General, Varint};
use crate::DecodeErrorKind::{InvalidValue, OutOfDomainValue};
use chrono::{Datelike, NaiveDate, NaiveTime, Timelike};

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
        let ordinal0: u32 = ordinal0.try_into().map_err(|_| InvalidValue)?;
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
        let ordinal0: u32 = ordinal0.try_into().map_err(|_| InvalidValue)?;
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
//  * struct DateTime<impl TimeZone>
//      * aggreagate of (NaiveDateTime, offset)
//      * store as tuple
//  * struct TimeDelta
//      * matches bilrost_types::Duration, but nanos is always positive
//      * use derived storage
