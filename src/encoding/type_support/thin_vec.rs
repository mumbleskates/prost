use crate::encoding::value_traits::{for_overwrite_via_default, TriviallyDistinguishedCollection};
use crate::encoding::{delegate_encoding, Collection, EmptyState, General, Unpacked};
use crate::DecodeErrorKind;

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

delegate_encoding!(delegate from (General) to (Unpacked<General>)
    for type (thin_vec::ThinVec<T>) including distinguished with generics (T));
