#[allow(unused_imports)]
use crate::{
    encoding::value_traits::{
        for_overwrite_via_default, Collection, EmptyState, ForOverwrite, Mapping,
        TriviallyDistinguishedCollection,
    },
    DecodeErrorKind::{self, InvalidValue, UnexpectedlyRepeated},
};

#[cfg(feature = "arrayvec")]
mod impl_arrayvec {
    use super::*;

    for_overwrite_via_default!(arrayvec::ArrayVec<T, N>, with generics (T, const N: usize));

    impl<T, const N: usize> EmptyState for arrayvec::ArrayVec<T, N> {
        #[inline]
        fn is_empty(&self) -> bool {
            arrayvec::ArrayVec::is_empty(self)
        }

        #[inline]
        fn clear(&mut self) {
            arrayvec::ArrayVec::clear(self)
        }
    }

    impl<T, const N: usize> Collection for arrayvec::ArrayVec<T, N> {
        type Item = T;
        type RefIter<'a>
            = core::slice::Iter<'a, T>
        where
            T: 'a,
            Self: 'a;
        type ReverseIter<'a>
            = core::iter::Rev<core::slice::Iter<'a, T>>
        where
            Self::Item: 'a,
            Self: 'a;

        #[inline]
        fn len(&self) -> usize {
            arrayvec::ArrayVec::len(self)
        }

        #[inline]
        fn iter(&self) -> Self::RefIter<'_> {
            self.as_slice().iter()
        }

        #[inline]
        fn reversed(&self) -> Self::ReverseIter<'_> {
            self.as_slice().iter().rev()
        }

        #[inline]
        fn insert(&mut self, item: Self::Item) -> Result<(), DecodeErrorKind> {
            self.try_push(item).map_err(|_| InvalidValue)
        }
    }

    impl<T, const N: usize> TriviallyDistinguishedCollection for arrayvec::ArrayVec<T, N> {}
}

#[cfg(feature = "bstr")]
mod impl_bstr {
    use super::*;
    use alloc::vec::Vec;

    for_overwrite_via_default!(bstr::BString);

    impl EmptyState for bstr::BString {
        #[inline]
        fn is_empty(&self) -> bool {
            Vec::is_empty(self)
        }

        #[inline]
        fn clear(&mut self) {
            Vec::clear(self)
        }
    }
}

#[cfg(feature = "bytestring")]
mod impl_bytestring {
    use super::*;

    for_overwrite_via_default!(bytestring::ByteString);

    impl EmptyState for bytestring::ByteString {
        #[inline]
        fn is_empty(&self) -> bool {
            str::is_empty(self)
        }

        #[inline]
        fn clear(&mut self) {
            *self = Self::empty();
        }
    }
}

#[cfg(feature = "chrono")]
mod impl_chrono {
    use super::*;

    impl ForOverwrite for chrono::NaiveDate {
        fn for_overwrite() -> Self {
            Self::from_yo_opt(0, 1).unwrap()
        }
    }

    impl EmptyState for chrono::NaiveDate {
        fn is_empty(&self) -> bool {
            use chrono::Datelike;
            (self.year(), self.ordinal0()) == (0, 0)
        }

        fn clear(&mut self) {
            *self = Self::empty();
        }
    }
}

#[cfg(feature = "hashbrown")]
mod impl_hashbrown {
    use super::*;

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
}

#[cfg(feature = "smallvec")]
mod impl_smallvec {
    use super::*;

    for_overwrite_via_default!(smallvec::SmallVec<A>,
        with generics(A),
        with where clause (A: smallvec::Array));

    impl<A: smallvec::Array> EmptyState for smallvec::SmallVec<A> {
        #[inline]
        fn is_empty(&self) -> bool {
            Self::is_empty(self)
        }

        #[inline]
        fn clear(&mut self) {
            Self::clear(self)
        }
    }

    impl<T, A: smallvec::Array<Item = T>> Collection for smallvec::SmallVec<A> {
        type Item = T;
        type RefIter<'a>
            = core::slice::Iter<'a, T>
        where
            T: 'a,
            Self: 'a;
        type ReverseIter<'a>
            = core::iter::Rev<core::slice::Iter<'a, T>>
        where
            Self::Item: 'a,
            Self: 'a;

        #[inline]
        fn len(&self) -> usize {
            smallvec::SmallVec::len(self)
        }

        #[inline]
        fn iter(&self) -> Self::RefIter<'_> {
            <[T]>::iter(self)
        }

        #[inline]
        fn reversed(&self) -> Self::ReverseIter<'_> {
            <[T]>::iter(self).rev()
        }

        #[inline]
        fn insert(&mut self, item: T) -> Result<(), DecodeErrorKind> {
            smallvec::SmallVec::push(self, item);
            Ok(())
        }
    }

    impl<A: smallvec::Array> TriviallyDistinguishedCollection for smallvec::SmallVec<A> {}
}

#[cfg(feature = "std")]
mod impl_std {
    use super::*;
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
}

#[cfg(feature = "thin-vec")]
mod impl_thin_vec {
    use super::*;

    for_overwrite_via_default!(thin_vec::ThinVec<T>, with generics (T));

    impl<T> EmptyState for thin_vec::ThinVec<T> {
        #[inline]
        fn is_empty(&self) -> bool {
            Self::is_empty(self)
        }

        #[inline]
        fn clear(&mut self) {
            Self::clear(self)
        }
    }

    impl<T> Collection for thin_vec::ThinVec<T> {
        type Item = T;
        type RefIter<'a>
            = core::slice::Iter<'a, T>
        where
            T: 'a,
            Self: 'a;
        type ReverseIter<'a>
            = core::iter::Rev<core::slice::Iter<'a, T>>
        where
            Self::Item: 'a,
            Self: 'a;

        #[inline]
        fn len(&self) -> usize {
            thin_vec::ThinVec::len(self)
        }

        #[inline]
        fn iter(&self) -> Self::RefIter<'_> {
            <[T]>::iter(self)
        }

        #[inline]
        fn reversed(&self) -> Self::ReverseIter<'_> {
            <[T]>::iter(self).rev()
        }

        #[inline]
        fn insert(&mut self, item: T) -> Result<(), DecodeErrorKind> {
            thin_vec::ThinVec::push(self, item);
            Ok(())
        }
    }

    impl<T> TriviallyDistinguishedCollection for thin_vec::ThinVec<T> {}
}

#[cfg(feature = "tinyvec")]
mod impl_tinyvec {
    use super::*;

    for_overwrite_via_default!(tinyvec::ArrayVec<A>,
        with generics (A),
        with where clause (A: tinyvec::Array));

    impl<A: tinyvec::Array> EmptyState for tinyvec::ArrayVec<A> {
        #[inline]
        fn is_empty(&self) -> bool {
            tinyvec::ArrayVec::is_empty(self)
        }

        #[inline]
        fn clear(&mut self) {
            tinyvec::ArrayVec::clear(self)
        }
    }

    impl<T, A: tinyvec::Array<Item = T>> Collection for tinyvec::ArrayVec<A> {
        type Item = T;
        type RefIter<'a>
            = core::slice::Iter<'a, T>
        where
            T: 'a,
            Self: 'a;
        type ReverseIter<'a>
            = core::iter::Rev<core::slice::Iter<'a, T>>
        where
            Self::Item: 'a,
            Self: 'a;

        #[inline]
        fn len(&self) -> usize {
            tinyvec::ArrayVec::len(self)
        }

        #[inline]
        fn iter(&self) -> Self::RefIter<'_> {
            self.as_slice().iter()
        }

        #[inline]
        fn reversed(&self) -> Self::ReverseIter<'_> {
            self.as_slice().iter().rev()
        }

        #[inline]
        fn insert(&mut self, item: Self::Item) -> Result<(), DecodeErrorKind> {
            match self.try_push(item) {
                None => Ok(()),
                Some(_) => Err(InvalidValue),
            }
        }
    }

    impl<A: tinyvec::Array> TriviallyDistinguishedCollection for tinyvec::ArrayVec<A> {}

    for_overwrite_via_default!(tinyvec::TinyVec<A>,
        with generics (A),
        with where clause (A: tinyvec::Array));

    impl<A: tinyvec::Array> EmptyState for tinyvec::TinyVec<A> {
        #[inline]
        fn is_empty(&self) -> bool {
            Self::is_empty(self)
        }

        #[inline]
        fn clear(&mut self) {
            Self::clear(self)
        }
    }

    impl<T, A: tinyvec::Array<Item = T>> Collection for tinyvec::TinyVec<A> {
        type Item = T;
        type RefIter<'a>
            = core::slice::Iter<'a, T>
        where
            T: 'a,
            Self: 'a;
        type ReverseIter<'a>
            = core::iter::Rev<core::slice::Iter<'a, T>>
        where
            Self::Item: 'a,
            Self: 'a;

        #[inline]
        fn len(&self) -> usize {
            tinyvec::TinyVec::len(self)
        }

        #[inline]
        fn iter(&self) -> Self::RefIter<'_> {
            <[T]>::iter(self)
        }

        #[inline]
        fn reversed(&self) -> Self::ReverseIter<'_> {
            <[T]>::iter(self).rev()
        }

        #[inline]
        fn insert(&mut self, item: T) -> Result<(), DecodeErrorKind> {
            tinyvec::TinyVec::push(self, item);
            Ok(())
        }
    }

    impl<A: tinyvec::Array> TriviallyDistinguishedCollection for tinyvec::TinyVec<A> {}
}
