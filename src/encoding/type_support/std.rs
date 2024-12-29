use core::cmp::Ordering;
use crate::encoding::value_traits::for_overwrite_via_default;
use crate::encoding::{delegate_encoding, delegate_value_encoding, Collection, EmptyState, ForOverwrite, General, Map, Mapping, Packed, Proxiable, Proxied, Unpacked, Varint};
use crate::DecodeErrorKind;
use crate::DecodeErrorKind::{InvalidValue, OutOfDomainValue, UnexpectedlyRepeated};
use std::collections::{hash_map, hash_set, HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

impl ForOverwrite for std::time::SystemTime {
    fn for_overwrite() -> Self {
        std::time::UNIX_EPOCH
    }
}

impl EmptyState for std::time::SystemTime {
    fn is_empty(&self) -> bool {
        *self == std::time::UNIX_EPOCH
    }

    fn clear(&mut self) {
        *self = std::time::UNIX_EPOCH;
    }
}

for_overwrite_via_default!(HashSet<T, S>,
    with generics (T, S),
    with where clause (S: Default + core::hash::BuildHasher));

impl<T, S> EmptyState for HashSet<T, S>
where
    S: Default + core::hash::BuildHasher,
{
    #[inline]
    fn is_empty(&self) -> bool {
        HashSet::is_empty(self)
    }

    #[inline]
    fn clear(&mut self) {
        HashSet::clear(self)
    }
}

impl<T, S> Collection for HashSet<T, S>
where
    T: Eq + core::hash::Hash,
    S: Default + core::hash::BuildHasher,
{
    type Item = T;
    type RefIter<'a>
    = hash_set::Iter<'a, T>
    where
        Self::Item: 'a,
        Self: 'a;
    type ReverseIter<'a>
    = Self::RefIter<'a>
    where
        Self::Item: 'a,
        Self: 'a;

    #[inline]
    fn len(&self) -> usize {
        HashSet::len(self)
    }

    #[inline]
    fn iter(&self) -> Self::RefIter<'_> {
        HashSet::iter(self)
    }

    #[inline]
    fn reversed(&self) -> Self::ReverseIter<'_> {
        HashSet::iter(self)
    }

    #[inline]
    fn insert(&mut self, item: Self::Item) -> Result<(), DecodeErrorKind> {
        if !HashSet::insert(self, item) {
            return Err(UnexpectedlyRepeated);
        }
        Ok(())
    }
}

for_overwrite_via_default!(HashMap<K, V, S>,
    with generics (K, V, S),
    with where clause (S: Default + core::hash::BuildHasher));

impl<K, V, S> EmptyState for HashMap<K, V, S>
where
    S: Default + core::hash::BuildHasher,
{
    #[inline]
    fn is_empty(&self) -> bool {
        HashMap::is_empty(self)
    }

    #[inline]
    fn clear(&mut self) {
        HashMap::clear(self)
    }
}

impl<K, V, S> Mapping for HashMap<K, V, S>
where
    K: Eq + core::hash::Hash,
    S: Default + core::hash::BuildHasher,
{
    type Key = K;
    type Value = V;
    type RefIter<'a>
    = hash_map::Iter<'a, K, V>
    where
        K: 'a,
        V: 'a,
        Self: 'a;
    type ReverseIter<'a>
    = Self::RefIter<'a>
    where
        K: 'a,
        V: 'a,
        Self: 'a;

    #[inline]
    fn len(&self) -> usize {
        HashMap::len(self)
    }

    #[inline]
    fn iter(&self) -> Self::RefIter<'_> {
        HashMap::iter(self)
    }

    #[inline]
    fn reversed(&self) -> Self::ReverseIter<'_> {
        HashMap::iter(self)
    }

    #[inline]
    fn insert(&mut self, key: K, value: V) -> Result<(), DecodeErrorKind> {
        if let hash_map::Entry::Vacant(entry) = self.entry(key) {
            entry.insert(value);
            Ok(())
        } else {
            Err(UnexpectedlyRepeated)
        }
    }
}

impl Proxiable for SystemTime {
    type Proxy = crate::encoding::local_proxy::LocalProxy<u64, 3>;
    fn new_proxy() -> Self::Proxy {
        Self::Proxy::new_empty()
    }

    fn encode_proxy(&self) -> Self::Proxy {
        let (symbol, small, big) = match self.cmp(&UNIX_EPOCH) {
            Ordering::Equal => {
                return Self::Proxy::new_empty();
            }
            Ordering::Greater => ('+', &UNIX_EPOCH, self),
            Ordering::Less => ('-', self, &UNIX_EPOCH),
        };
        let magnitude = big
            .duration_since(*small)
            .expect("SystemTime dates ordered wrong");
        Self::Proxy::new_without_empty_suffix([
            symbol as u64,
            magnitude.as_secs(),
            magnitude.subsec_nanos() as u64,
        ])
    }

    fn decode_proxy(&mut self, proxy: Self::Proxy) -> Result<(), DecodeErrorKind> {
        let (operation, secs, nanos): (fn(_, _) -> _, u64, u64) = match proxy.into_inner() {
            [0, 0, 0] => {
                *self = UNIX_EPOCH;
                return Ok(());
            }
            [symbol, secs, nanos] if symbol == '+' as u64 => (SystemTime::checked_add, secs, nanos),
            [symbol, secs, nanos] if symbol == '-' as u64 => (SystemTime::checked_sub, secs, nanos),
            _ => return Err(InvalidValue),
        };
        let nanos = nanos
            .try_into()
            .map_err(|_| InvalidValue)
            .and_then(|nanos| {
                if nanos > 999_999_999 {
                    Err(InvalidValue)
                } else {
                    Ok(nanos)
                }
            })?;
        *self = operation(&UNIX_EPOCH, core::time::Duration::new(secs, nanos)).ok_or(OutOfDomainValue)?;
        Ok(())
    }
}

delegate_value_encoding!(delegate from (General) to (Proxied<Packed<Varint>>)
    for type (SystemTime));

#[cfg(test)]
mod systemtime {
    use super::*;
    use crate::encoding::test::{check_type_empty, check_type_test};

    check_type_empty!(SystemTime, via proxy);
    check_type_test!(General, expedient, SystemTime, WireType::LengthDelimited);
}

delegate_encoding!(delegate from (General) to (Unpacked<General>)
    for type (HashSet<T, S>)
    with where clause (S: Default + core::hash::BuildHasher)
    with generics (T, S));
delegate_value_encoding!(delegate from (General) to (Map<General, General>)
    for type (HashMap<K, V, S>)
    with where clause (K: Eq + core::hash::Hash, S: Default + core::hash::BuildHasher)
    with generics (K, V, S));

#[cfg(test)]
mod test {
    mod hash_map {
        mod general {
            use crate::encoding::test::check_type_test;
            use crate::encoding::{General, Map};
            use std::collections::HashMap;
            check_type_test!(
                Map<General, General>,
                expedient,
                HashMap<u64, f32>,
                WireType::LengthDelimited
            );
        }

        mod fixed {
            use crate::encoding::test::check_type_test;
            use crate::encoding::{Fixed, Map};
            use std::collections::HashMap;
            check_type_test!(
                Map<Fixed, Fixed>,
                expedient,
                HashMap<u64, f32>,
                WireType::LengthDelimited
            );
        }

        mod delegated_from_general {
            use crate::encoding::test::check_type_test;
            use crate::encoding::General;
            use std::collections::HashMap;
            check_type_test!(
                General,
                expedient,
                HashMap<bool, u32>,
                WireType::LengthDelimited
            );
        }
    }
}
