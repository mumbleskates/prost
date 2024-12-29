use crate::encoding::proxy_encoder;
use crate::encoding::{Canonicity, DecodeErrorKind, EmptyState, ForOverwrite, General, Varint};
use crate::DecodeErrorKind::{InvalidValue, OutOfDomainValue};
use chrono::Datelike;

impl ForOverwrite for chrono::NaiveDate {
    fn for_overwrite() -> Self {
        Self::from_yo_opt(0, 1).unwrap()
    }
}

impl EmptyState for chrono::NaiveDate {
    fn is_empty(&self) -> bool {
        (self.year(), self.ordinal0()) == (0, 0)
    }

    fn clear(&mut self) {
        *self = Self::empty();
    }
}

mod naivedate {
    use super::*;
    use chrono::NaiveDate;

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

    fn from_proxy_distinguished(proxy: Proxy) -> Result<(NaiveDate, Canonicity), DecodeErrorKind> {
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
        use alloc::vec::Vec;

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
