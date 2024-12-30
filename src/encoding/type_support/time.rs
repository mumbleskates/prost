use crate::encoding::local_proxy::LocalProxy;
use crate::encoding::{
    delegate_value_encoding, Canonicity, DecodeErrorKind, DistinguishedProxiable, EmptyState,
    ForOverwrite, General, Packed, Proxiable, Proxied, Varint,
};
use crate::DecodeErrorKind::OutOfDomainValue;
use time::{Date, Time};

#[cfg(test)]
const RANDOM_SAMPLES: u32 = 100;

impl ForOverwrite for Date {
    fn for_overwrite() -> Self {
        Self::from_ordinal_date(0, 1).unwrap()
    }
}

impl EmptyState for Date {
    fn is_empty(&self) -> bool {
        *self == Self::empty()
    }

    fn clear(&mut self) {
        *self = Self::empty();
    }
}

impl Proxiable for Date {
    type Proxy = LocalProxy<i32, 2>;

    fn new_proxy() -> Self::Proxy {
        Self::Proxy::new_empty()
    }

    fn encode_proxy(&self) -> Self::Proxy {
        Self::Proxy::new_without_empty_suffix([self.year(), (self.ordinal() - 1) as i32])
    }

    fn decode_proxy(&mut self, proxy: Self::Proxy) -> Result<(), DecodeErrorKind> {
        let [year, ordinal0] = proxy.into_inner();
        let ordinal0: Option<u16> = ordinal0.try_into().ok();
        let ordinal = ordinal0
            .and_then(|o| o.checked_add(1))
            .ok_or(OutOfDomainValue)?;
        *self = Self::from_ordinal_date(year, ordinal).map_err(|_| OutOfDomainValue)?;
        Ok(())
    }
}

impl DistinguishedProxiable for Date {
    fn decode_proxy_distinguished(
        &mut self,
        proxy: Self::Proxy,
    ) -> Result<Canonicity, DecodeErrorKind> {
        let ([year, ordinal0], canon) = proxy.into_inner_distinguished();
        let ordinal0: Option<u16> = ordinal0.try_into().ok();
        let ordinal = ordinal0
            .and_then(|o| o.checked_add(1))
            .ok_or(OutOfDomainValue)?;
        *self = Self::from_ordinal_date(year, ordinal).map_err(|_| OutOfDomainValue)?;
        Ok(canon)
    }
}

delegate_value_encoding!(delegate from (General) to (Proxied<Packed<Varint>>)
    for type (Date) including distinguished);

#[cfg(test)]
mod date {
    use super::RANDOM_SAMPLES;
    use crate::encoding::test::check_type_empty;
    use crate::encoding::test::{distinguished, expedient};
    use crate::encoding::{EmptyState, WireType};
    use rand::{thread_rng, Rng};
    use time::Date;
    use time::Month::{January, June};

    #[test]
    fn check_type() {
        let mut rng = thread_rng();

        for date in [
            Date::MIN,
            Date::MAX,
            Date::empty(),
            Date::from_calendar_date(1970, January, 1).unwrap(),
            Date::from_calendar_date(1998, June, 28).unwrap(),
        ] {
            expedient::check_type(date, 123, WireType::LengthDelimited).unwrap();
            distinguished::check_type(date, 123, WireType::LengthDelimited).unwrap();
        }

        for i in 0..RANDOM_SAMPLES {
            let date: Date = rng.gen();
            expedient::check_type(date, i, WireType::LengthDelimited).unwrap();
            distinguished::check_type(date, i, WireType::LengthDelimited).unwrap();
        }
    }
    check_type_empty!(Date, via proxy);
    check_type_empty!(Date, via distinguished proxy);
}

impl ForOverwrite for Time {
    fn for_overwrite() -> Self {
        Time::MIDNIGHT
    }
}

impl EmptyState for Time {
    fn is_empty(&self) -> bool {
        *self == Time::MIDNIGHT
    }

    fn clear(&mut self) {
        *self = Time::MIDNIGHT;
    }
}

#[inline(always)]
fn parts_to_time<T>(hour: T, min: T, sec: T, nano: T) -> Option<Time>
where
    T: Into<u32>,
    u8: TryFrom<T>,
{
    Time::from_hms_nano(
        u8::try_from(hour).ok()?,
        u8::try_from(min).ok()?,
        u8::try_from(sec).ok()?,
        nano.into(),
    )
    .ok()
}

impl Proxiable for Time {
    type Proxy = LocalProxy<u32, 4>;

    fn new_proxy() -> Self::Proxy {
        Self::Proxy::new_empty()
    }

    fn encode_proxy(&self) -> Self::Proxy {
        Self::Proxy::new_without_empty_suffix([
            self.hour() as u32,
            self.minute() as u32,
            self.second() as u32,
            self.nanosecond(),
        ])
    }

    fn decode_proxy(&mut self, proxy: Self::Proxy) -> Result<(), DecodeErrorKind> {
        let [hour, min, sec, nano] = proxy.into_inner();
        *self = parts_to_time(hour, min, sec, nano).ok_or(OutOfDomainValue)?;
        Ok(())
    }
}

impl DistinguishedProxiable for Time {
    fn decode_proxy_distinguished(
        &mut self,
        proxy: Self::Proxy,
    ) -> Result<Canonicity, DecodeErrorKind> {
        let ([hour, min, sec, nano], canon) = proxy.into_inner_distinguished();
        *self = parts_to_time(hour, min, sec, nano).ok_or(OutOfDomainValue)?;
        Ok(canon)
    }
}

delegate_value_encoding!(delegate from (General) to (Proxied<Packed<Varint>>)
    for type (Time) including distinguished);

#[cfg(test)]
mod time_ty {
    use super::RANDOM_SAMPLES;
    use crate::encoding::test::check_type_empty;
    use crate::encoding::test::{distinguished, expedient};
    use crate::encoding::{EmptyState, WireType};
    use rand::{thread_rng, Rng};
    use time::Time;

    #[test]
    fn check_type() {
        let mut rng = thread_rng();

        for date in [
            Time::MIDNIGHT,
            Time::MAX,
            Time::empty(),
            Time::from_hms_nano(11, 11, 11, 111_111_111).unwrap(),
        ] {
            expedient::check_type(date, 123, WireType::LengthDelimited).unwrap();
            distinguished::check_type(date, 123, WireType::LengthDelimited).unwrap();
        }

        for i in 0..RANDOM_SAMPLES {
            let time: Time = rng.gen();
            expedient::check_type(time, i, WireType::LengthDelimited).unwrap();
            distinguished::check_type(time, i, WireType::LengthDelimited).unwrap();
        }
    }
    check_type_empty!(Time, via proxy);
    check_type_empty!(Time, via distinguished proxy);
}

// TODO(widders): this
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
