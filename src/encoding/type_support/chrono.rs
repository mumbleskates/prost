use crate::encoding::{
    delegate_value_encoding, DistinguishedProxiable, Packed, Proxiable, Proxied,
};
use crate::encoding::{Canonicity, DecodeErrorKind, EmptyState, ForOverwrite, General, Varint};
use crate::DecodeErrorKind::{InvalidValue, OutOfDomainValue};
use chrono::{Datelike, NaiveDate};

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
    type Proxy = crate::encoding::local_proxy::LocalProxy<i32, 2>;

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
    use super::*;
    use crate::encoding::test::{check_type_empty, check_type_test};
    use alloc::vec::Vec;

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
