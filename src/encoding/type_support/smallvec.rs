use crate::encoding::value_traits::{for_overwrite_via_default, TriviallyDistinguishedCollection};
use crate::encoding::{delegate_encoding, Collection, EmptyState, General, Unpacked};
use crate::DecodeErrorKind;

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

delegate_encoding!(delegate from (General) to (Unpacked<General>)
    for type (smallvec::SmallVec<A>) including distinguished
    with where clause (A: smallvec::Array<Item = T>)
    with generics (T, A));
