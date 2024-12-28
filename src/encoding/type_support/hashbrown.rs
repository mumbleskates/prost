use crate::encoding::value_traits::for_overwrite_via_default;
use crate::encoding::{
    delegate_encoding, delegate_value_encoding, Collection, EmptyState, General, Map, Mapping,
    Unpacked,
};
use crate::DecodeErrorKind;
use crate::DecodeErrorKind::UnexpectedlyRepeated;

for_overwrite_via_default!(hashbrown::HashSet<T, S>,
        with generics (T, S),
        with where clause (S: Default + core::hash::BuildHasher));

impl<T, S> EmptyState for hashbrown::HashSet<T, S>
where
    S: Default + core::hash::BuildHasher,
{
    #[inline]
    fn is_empty(&self) -> bool {
        hashbrown::HashSet::is_empty(self)
    }

    #[inline]
    fn clear(&mut self) {
        hashbrown::HashSet::clear(self)
    }
}

impl<T, S> Collection for hashbrown::HashSet<T, S>
where
    T: Eq + core::hash::Hash,
    S: Default + core::hash::BuildHasher,
{
    type Item = T;
    type RefIter<'a>
        = hashbrown::hash_set::Iter<'a, T>
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
        hashbrown::HashSet::len(self)
    }

    #[inline]
    fn iter(&self) -> Self::RefIter<'_> {
        hashbrown::HashSet::iter(self)
    }

    #[inline]
    fn reversed(&self) -> Self::ReverseIter<'_> {
        hashbrown::HashSet::iter(self)
    }

    #[inline]
    fn insert(&mut self, item: Self::Item) -> Result<(), DecodeErrorKind> {
        if !hashbrown::HashSet::insert(self, item) {
            return Err(UnexpectedlyRepeated);
        }
        Ok(())
    }
}

for_overwrite_via_default!(hashbrown::HashMap<K, V, S>,
        with generics (K, V, S),
        with where clause (S: Default + core::hash::BuildHasher));

impl<K, V, S> EmptyState for hashbrown::HashMap<K, V, S>
where
    S: Default + core::hash::BuildHasher,
{
    #[inline]
    fn is_empty(&self) -> bool {
        hashbrown::HashMap::is_empty(self)
    }

    #[inline]
    fn clear(&mut self) {
        hashbrown::HashMap::clear(self)
    }
}

impl<K, V, S> Mapping for hashbrown::HashMap<K, V, S>
where
    K: Eq + core::hash::Hash,
    S: Default + core::hash::BuildHasher,
{
    type Key = K;
    type Value = V;
    type RefIter<'a>
        = hashbrown::hash_map::Iter<'a, K, V>
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
        hashbrown::HashMap::len(self)
    }

    #[inline]
    fn iter(&self) -> Self::RefIter<'_> {
        hashbrown::HashMap::iter(self)
    }

    #[inline]
    fn reversed(&self) -> Self::ReverseIter<'_> {
        hashbrown::HashMap::iter(self)
    }

    #[inline]
    fn insert(&mut self, key: K, value: V) -> Result<(), DecodeErrorKind> {
        if let hashbrown::hash_map::Entry::Vacant(entry) = self.entry(key) {
            entry.insert(value);
            Ok(())
        } else {
            Err(UnexpectedlyRepeated)
        }
    }
}

delegate_encoding!(delegate from (General) to (Unpacked<General>)
    for type (hashbrown::HashSet<T, S>)
    with where clause (S: Default + core::hash::BuildHasher)
    with generics (T, S));
delegate_value_encoding!(delegate from (General) to (Map<General, General>)
    for type (hashbrown::HashMap<K, V, S>)
    with where clause (K: Eq + core::hash::Hash, S: Default + core::hash::BuildHasher)
    with generics (K, V, S));
