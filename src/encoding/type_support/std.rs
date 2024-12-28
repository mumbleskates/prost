use crate::encoding::value_traits::for_overwrite_via_default;
use crate::encoding::{
    delegate_encoding, delegate_value_encoding, Collection, EmptyState, ForOverwrite, General, Map,
    Mapping, Unpacked,
};
use crate::DecodeErrorKind;
use crate::DecodeErrorKind::UnexpectedlyRepeated;
use std::collections::{hash_map, hash_set, HashMap, HashSet};

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

delegate_encoding!(delegate from (General) to (Unpacked<General>)
    for type (HashSet<T, S>)
    with where clause (S: Default + core::hash::BuildHasher)
    with generics (T, S));
delegate_value_encoding!(delegate from (General) to (Map<General, General>)
    for type (HashMap<K, V, S>)
    with where clause (K: Eq + core::hash::Hash, S: Default + core::hash::BuildHasher)
    with generics (K, V, S));
