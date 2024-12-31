use crate::encoding::local_proxy::LocalProxy;
use crate::encoding::{
    delegate_value_encoding, Canonicity, DecodeErrorKind, DistinguishedProxiable, EmptyState,
    ForOverwrite, General, Packed, Proxiable, Proxied, Varint,
};
use crate::Canonicity::Canonical;
use crate::DecodeErrorKind::{InvalidValue, OutOfDomainValue};
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

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

#[inline(always)]
fn parts_to_date(year: i32, ordinal0: i32) -> Option<Date> {
    let ordinal = u16::try_from(ordinal0)
        .ok()
        .and_then(|o| o.checked_add(1))?;
    Date::from_ordinal_date(year, ordinal).ok()
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
        *self = parts_to_date(year, ordinal0).ok_or(OutOfDomainValue)?;
        Ok(())
    }
}

impl DistinguishedProxiable for Date {
    fn decode_proxy_distinguished(
        &mut self,
        proxy: Self::Proxy,
    ) -> Result<Canonicity, DecodeErrorKind> {
        let ([year, ordinal0], canon) = proxy.into_inner_distinguished();
        *self = parts_to_date(year, ordinal0).ok_or(OutOfDomainValue)?;
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

    pub(super) fn test_dates() -> impl Iterator<Item = Date> + Clone {
        [
            Date::MIN,
            Date::MAX,
            Date::empty(),
            Date::from_calendar_date(1970, January, 1).unwrap(),
            Date::from_calendar_date(1998, June, 28).unwrap(),
        ]
        .into_iter()
    }

    #[test]
    fn check_type() {
        let mut rng = thread_rng();

        for date in test_dates() {
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
    T: TryInto<u8> + TryInto<u32>,
{
    Time::from_hms_nano(
        hour.try_into().ok()?,
        min.try_into().ok()?,
        sec.try_into().ok()?,
        nano.try_into().ok()?,
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

    pub(super) fn test_times() -> impl Iterator<Item = Time> + Clone {
        [
            Time::MIDNIGHT,
            Time::MAX,
            Time::empty(),
            Time::from_hms_nano(11, 11, 11, 111_111_111).unwrap(),
        ]
        .into_iter()
    }

    #[test]
    fn check_type() {
        let mut rng = thread_rng();

        for date in test_times() {
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

impl ForOverwrite for PrimitiveDateTime {
    fn for_overwrite() -> Self {
        Self::new(EmptyState::empty(), EmptyState::empty())
    }
}

impl EmptyState for PrimitiveDateTime {
    fn is_empty(&self) -> bool {
        self.date().is_empty() && self.time().is_empty()
    }

    fn clear(&mut self) {
        *self = Self::empty();
    }
}

impl Proxiable for PrimitiveDateTime {
    type Proxy = LocalProxy<i32, 6>;

    fn new_proxy() -> Self::Proxy {
        Self::Proxy::new_empty()
    }

    fn encode_proxy(&self) -> Self::Proxy {
        Self::Proxy::new_without_empty_suffix([
            self.year(),
            (self.ordinal() - 1) as i32,
            self.hour() as i32,
            self.minute() as i32,
            self.second() as i32,
            self.nanosecond() as i32,
        ])
    }

    fn decode_proxy(&mut self, proxy: Self::Proxy) -> Result<(), DecodeErrorKind> {
        let [year, ordinal0, hour, min, sec, nano] = proxy.into_inner();
        *self = Self::new(
            parts_to_date(year, ordinal0).ok_or(OutOfDomainValue)?,
            parts_to_time(hour, min, sec, nano).ok_or(OutOfDomainValue)?,
        );
        Ok(())
    }
}

impl DistinguishedProxiable for PrimitiveDateTime {
    fn decode_proxy_distinguished(
        &mut self,
        proxy: Self::Proxy,
    ) -> Result<Canonicity, DecodeErrorKind> {
        let ([year, ordinal0, hour, min, sec, nano], canon) = proxy.into_inner_distinguished();
        *self = Self::new(
            parts_to_date(year, ordinal0).ok_or(OutOfDomainValue)?,
            parts_to_time(hour, min, sec, nano).ok_or(OutOfDomainValue)?,
        );
        Ok(canon)
    }
}

delegate_value_encoding!(delegate from (General) to (Proxied<Packed<Varint>>)
    for type (PrimitiveDateTime) including distinguished);

#[cfg(test)]
mod primitivedatetime {
    use super::date::test_dates;
    use super::time_ty::test_times;
    use super::RANDOM_SAMPLES;
    use crate::encoding::test::check_type_empty;
    use crate::encoding::test::{distinguished, expedient};
    use crate::encoding::{EmptyState, WireType};
    use itertools::iproduct;
    use rand::{thread_rng, Rng};
    use time::Month::{August, March};
    use time::{Date, PrimitiveDateTime, Time};

    pub(super) fn test_datetimes() -> impl IntoIterator<Item = PrimitiveDateTime> {
        [
            PrimitiveDateTime::MIN,
            PrimitiveDateTime::MAX,
            PrimitiveDateTime::empty(),
            PrimitiveDateTime::new(
                Date::from_calendar_date(-44, March, 15).unwrap(),
                Time::from_hms(12, 36, 27).unwrap(),
            ),
            PrimitiveDateTime::new(
                Date::from_calendar_date(-1753, August, 21).unwrap(),
                Time::from_hms(16, 49, 8).unwrap(),
            ),
        ]
        .into_iter()
        .chain(
            iproduct!(test_dates(), test_times())
                .map(|(date, time)| PrimitiveDateTime::new(date, time)),
        )
    }

    #[test]
    fn check_type() {
        let mut rng = thread_rng();

        for date in test_datetimes() {
            expedient::check_type(date, 123, WireType::LengthDelimited).unwrap();
            distinguished::check_type(date, 123, WireType::LengthDelimited).unwrap();
        }

        for i in 0..RANDOM_SAMPLES {
            let date: PrimitiveDateTime = rng.gen();
            expedient::check_type(date, i, WireType::LengthDelimited).unwrap();
            distinguished::check_type(date, i, WireType::LengthDelimited).unwrap();
        }
    }
    check_type_empty!(PrimitiveDateTime, via proxy);
    check_type_empty!(PrimitiveDateTime, via distinguished proxy);
}

impl ForOverwrite for UtcOffset {
    fn for_overwrite() -> Self {
        Self::UTC
    }
}

impl EmptyState for UtcOffset {
    fn is_empty(&self) -> bool {
        *self == Self::UTC
    }

    fn clear(&mut self) {
        *self = Self::UTC;
    }
}

impl Proxiable for UtcOffset {
    type Proxy = (i8, i8, i8);

    fn new_proxy() -> Self::Proxy {
        (0, 0, 0)
    }

    fn encode_proxy(&self) -> Self::Proxy {
        self.as_hms()
    }

    fn decode_proxy(&mut self, proxy: Self::Proxy) -> Result<(), DecodeErrorKind> {
        let (hours, mins, secs) = proxy;

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

        *self = Self::from_hms(hours, mins, secs).map_err(|_| OutOfDomainValue)?;
        Ok(())
    }
}

impl DistinguishedProxiable for UtcOffset {
    fn decode_proxy_distinguished(
        &mut self,
        proxy: Self::Proxy,
    ) -> Result<Canonicity, DecodeErrorKind> {
        self.decode_proxy(proxy)?;
        Ok(Canonical)
    }
}

delegate_value_encoding!(delegate from (General) to (Proxied<(Varint, Varint, Varint)>)
    for type (UtcOffset) including distinguished);

#[cfg(test)]
mod utcoffset {
    use super::RANDOM_SAMPLES;
    use crate::encoding::test::{check_type_empty, distinguished, expedient};
    use crate::encoding::{
        Capped, DecodeContext, DistinguishedValueEncoder, EmptyState, ForOverwrite, General,
        ValueEncoder, WireType,
    };
    use crate::DecodeError;
    use crate::DecodeErrorKind::InvalidValue;
    use alloc::vec::Vec;
    use rand::{thread_rng, Rng};
    use time::UtcOffset;

    pub(super) fn test_zones() -> impl Iterator<Item = UtcOffset> + Clone {
        [
            UtcOffset::UTC,
            UtcOffset::empty(),
            UtcOffset::from_hms(-7, 15, 0).unwrap(),
            UtcOffset::from_hms(14, 0, 0).unwrap(),
        ]
        .into_iter()
    }

    #[test]
    fn check_type() {
        let mut rng = thread_rng();

        for date in test_zones() {
            expedient::check_type(date, 123, WireType::LengthDelimited).unwrap();
            distinguished::check_type(date, 123, WireType::LengthDelimited).unwrap();
        }

        for i in 0..RANDOM_SAMPLES {
            let date: UtcOffset = rng.gen();
            expedient::check_type(date, i, WireType::LengthDelimited).unwrap();
            distinguished::check_type(date, i, WireType::LengthDelimited).unwrap();
        }
    }
    check_type_empty!(UtcOffset, via proxy);
    check_type_empty!(UtcOffset, via distinguished proxy);

    #[test]
    fn utcoffset_rejects_mixed_signs() {
        {
            let mut buf = Vec::new();
            let out_of_range: (i32, i32, i32) = (10, 0, -10);
            ValueEncoder::<General>::encode_value(&out_of_range, &mut buf);
            let mut utc_off = UtcOffset::for_overwrite();
            assert_eq!(
                ValueEncoder::<General>::decode_value(
                    &mut utc_off,
                    Capped::new(&mut buf.as_slice()),
                    DecodeContext::default()
                ),
                Err(DecodeError::new(InvalidValue))
            );
            assert_eq!(
                DistinguishedValueEncoder::<General>::decode_value_distinguished::<true>(
                    &mut utc_off,
                    Capped::new(&mut buf.as_slice()),
                    DecodeContext::default()
                ),
                Err(DecodeError::new(InvalidValue))
            );
        }
    }
}

impl ForOverwrite for OffsetDateTime {
    fn for_overwrite() -> Self {
        Self::new_utc(EmptyState::empty(), EmptyState::empty())
    }
}

impl EmptyState for OffsetDateTime {
    fn is_empty(&self) -> bool {
        self.date().is_empty() && self.time().is_empty() && self.offset().is_empty()
    }

    fn clear(&mut self) {
        *self = Self::empty();
    }
}

impl Proxiable for OffsetDateTime {
    type Proxy = (PrimitiveDateTime, UtcOffset);

    fn new_proxy() -> Self::Proxy {
        EmptyState::empty()
    }

    fn encode_proxy(&self) -> Self::Proxy {
        (
            PrimitiveDateTime::new(self.date(), self.time()),
            self.offset(),
        )
    }

    fn decode_proxy(&mut self, proxy: Self::Proxy) -> Result<(), DecodeErrorKind> {
        let (datetime, offset) = proxy;
        *self = Self::new_in_offset(datetime.date(), datetime.time(), offset);
        Ok(())
    }
}

impl DistinguishedProxiable for OffsetDateTime {
    fn decode_proxy_distinguished(
        &mut self,
        proxy: Self::Proxy,
    ) -> Result<Canonicity, DecodeErrorKind> {
        self.decode_proxy(proxy)?;
        Ok(Canonical)
    }
}

delegate_value_encoding!(delegate from (General) to (Proxied<General>)
    for type (OffsetDateTime) including distinguished);

#[cfg(test)]
mod offsetdatetime {
    use super::primitivedatetime::test_datetimes;
    use super::utcoffset::test_zones;
    use super::RANDOM_SAMPLES;
    use crate::encoding::test::check_type_empty;
    use crate::encoding::test::{distinguished, expedient};
    use crate::encoding::WireType;
    use itertools::iproduct;
    use rand::{thread_rng, Rng};
    use time::OffsetDateTime;

    #[test]
    fn check_type() {
        let mut rng = thread_rng();

        for (datetime, zone) in iproduct!(test_datetimes(), test_zones()) {
            let odt = OffsetDateTime::new_in_offset(datetime.date(), datetime.time(), zone);
            expedient::check_type(odt, 123, WireType::LengthDelimited).unwrap();
            distinguished::check_type(odt, 123, WireType::LengthDelimited).unwrap();
        }

        for i in 0..RANDOM_SAMPLES {
            let odt: OffsetDateTime = rng.gen();
            expedient::check_type(odt, i, WireType::LengthDelimited).unwrap();
            distinguished::check_type(odt, i, WireType::LengthDelimited).unwrap();
        }
    }
    check_type_empty!(OffsetDateTime, via proxy);
    check_type_empty!(OffsetDateTime, via distinguished proxy);
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
