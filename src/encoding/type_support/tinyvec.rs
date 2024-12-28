use crate::encoding::value_traits::{for_overwrite_via_default, TriviallyDistinguishedCollection};
use crate::encoding::{delegate_encoding, Collection, EmptyState, General, Unpacked};
use crate::DecodeErrorKind;
use crate::DecodeErrorKind::InvalidValue;

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

delegate_encoding!(delegate from (General) to (Unpacked<General>)
    for type (tinyvec::ArrayVec<A>) including distinguished
    with where clause (A: tinyvec::Array<Item = T>)
    with generics (T, A));
delegate_encoding!(delegate from (General) to (Unpacked<General>)
    for type (tinyvec::TinyVec<A>) including distinguished
    with where clause (A: tinyvec::Array<Item = T>)
    with generics (T, A));
