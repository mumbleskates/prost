use crate::encoding::value_traits::{
    Collection, EmptyState, ForOverwrite, TriviallyDistinguishedCollection,
};
use crate::Canonicity::{Canonical, NotCanonical};
use crate::{Canonicity, DecodeErrorKind};

/// This type is a locally implemented stand-in for types like tinyvec::ArrayVec with bare-minimum
/// functionality to assist encoding some third party types.
pub(crate) struct LocalProxy<T, const N: usize> {
    arr: [T; N],
    size: usize,
}

impl<T: EmptyState, const N: usize> LocalProxy<T, N> {
    pub fn new() -> Self {
        Self::empty()
    }

    pub fn from_inner(arr: [T; N]) -> Self {
        Self { arr, size: N }
    }

    /// Removes all empty items from the end of the inner array.
    pub fn trim_empty_suffix(mut self) -> Self {
        for last_item in self.arr[..self.size].iter().rev() {
            if last_item.is_empty() {
                self.size -= 1;
            } else {
                break;
            }
        }
        self
    }

    pub fn into_inner(self) -> [T; N] {
        self.arr
    }

    pub fn into_inner_distinguished(self) -> ([T; N], Canonicity) {
        let canon = if self.reversed().next().is_some_and(EmptyState::is_empty) {
            NotCanonical
        } else {
            Canonical
        };
        (self.arr, canon)
    }
}

impl<T: EmptyState + PartialEq, const N: usize> PartialEq for LocalProxy<T, N> {
    fn eq(&self, other: &Self) -> bool {
        self.arr[..self.size] == other.arr[..other.size]
    }
}

impl<T: EmptyState + Eq, const N: usize> Eq for LocalProxy<T, N> {}

impl<T: EmptyState, const N: usize> ForOverwrite for LocalProxy<T, N> {
    fn for_overwrite() -> Self {
        Self {
            arr: core::array::from_fn(|_| EmptyState::empty()),
            size: 0,
        }
    }
}

impl<T: EmptyState, const N: usize> EmptyState for LocalProxy<T, N> {
    fn is_empty(&self) -> bool {
        self.size == 0
    }

    fn clear(&mut self) {
        self.size = 0;
    }
}

impl<T: EmptyState, const N: usize> Collection for LocalProxy<T, N> {
    type Item = T;
    type RefIter<'a>
        = core::slice::Iter<'a, T>
    where
        Self::Item: 'a,
        Self: 'a;
    type ReverseIter<'a>
        = core::iter::Rev<core::slice::Iter<'a, T>>
    where
        Self::Item: 'a,
        Self: 'a;

    fn len(&self) -> usize {
        self.size
    }

    fn iter(&self) -> Self::RefIter<'_> {
        self.arr[..self.size].iter()
    }

    fn reversed(&self) -> Self::ReverseIter<'_> {
        self.arr[..self.size].iter().rev()
    }

    fn insert(&mut self, item: Self::Item) -> Result<(), DecodeErrorKind> {
        if self.size == N {
            return Err(DecodeErrorKind::InvalidValue);
        }
        self.arr[self.size] = item;
        self.size += 1;
        Ok(())
    }
}

impl<T: EmptyState, const N: usize> TriviallyDistinguishedCollection for LocalProxy<T, N> {}
