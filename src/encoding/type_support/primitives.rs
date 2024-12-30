use crate::encoding::value_traits::empty_state_via_default;
use crate::encoding::{EmptyState, ForOverwrite};

empty_state_via_default!(bool);
empty_state_via_default!(u8);
empty_state_via_default!(u16);
empty_state_via_default!(u32);
empty_state_via_default!(u64);
empty_state_via_default!(usize);
empty_state_via_default!(i8);
empty_state_via_default!(i16);
empty_state_via_default!(i32);
empty_state_via_default!(i64);
empty_state_via_default!(isize);

macro_rules! empty_state_for_float {
    ($ty:ty) => {
        impl ForOverwrite for $ty {
            #[inline]
            fn for_overwrite() -> Self {
                0.0
            }
        }

        impl EmptyState for $ty {
            #[inline]
            fn is_empty(&self) -> bool {
                // Preserve -0.0. This is actually the original motivation for `EmptyState`.
                self.to_bits() == 0
            }

            #[inline]
            fn clear(&mut self) {
                *self = Self::empty();
            }
        }
    };
}
empty_state_for_float!(f32);
empty_state_for_float!(f64);

macro_rules! impls_for_tuple {
    (($($letters:ident),*), ($($numbers:tt),*)$(,)?) => {
        impl<$($letters,)*> ForOverwrite for ($($letters,)*)
        where
            $($letters: ForOverwrite,)*
        {
            #[inline]
            fn for_overwrite() -> Self {
                ($($letters::for_overwrite(),)*)
            }
        }

        impl<$($letters,)*> EmptyState for ($($letters,)*)
        where
            $($letters: EmptyState,)*
        {
            #[inline]
            fn empty() -> Self {
                ($($letters::empty(),)*)
            }

            #[inline]
            fn is_empty(&self) -> bool {
                true $(&& self.$numbers.is_empty())*
            }

            #[inline]
            fn clear(&mut self) {
                $(self.$numbers.clear();)*
            }
        }
    };
}
impls_for_tuple!((A), (0));
impls_for_tuple!((A, B), (0, 1));
impls_for_tuple!((A, B, C), (0, 1, 2));
impls_for_tuple!((A, B, C, D), (0, 1, 2, 3));
impls_for_tuple!((A, B, C, D, E), (0, 1, 2, 3, 4));
impls_for_tuple!((A, B, C, D, E, F), (0, 1, 2, 3, 4, 5));
impls_for_tuple!((A, B, C, D, E, F, G), (0, 1, 2, 3, 4, 5, 6));
impls_for_tuple!((A, B, C, D, E, F, G, H), (0, 1, 2, 3, 4, 5, 6, 7));
impls_for_tuple!((A, B, C, D, E, F, G, H, I), (0, 1, 2, 3, 4, 5, 6, 7, 8));
impls_for_tuple!(
    (A, B, C, D, E, F, G, H, I, J),
    (0, 1, 2, 3, 4, 5, 6, 7, 8, 9)
);
impls_for_tuple!(
    (A, B, C, D, E, F, G, H, I, J, K),
    (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
);
impls_for_tuple!(
    (A, B, C, D, E, F, G, H, I, J, K, L),
    (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)
);

impl<T, const N: usize> ForOverwrite for [T; N]
where
    T: ForOverwrite,
{
    #[inline]
    fn for_overwrite() -> Self {
        core::array::from_fn(|_| T::for_overwrite())
    }
}

impl<T, const N: usize> EmptyState for [T; N]
where
    T: EmptyState,
{
    #[inline]
    fn empty() -> Self
    where
        Self: Sized,
    {
        core::array::from_fn(|_| T::empty())
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.iter().all(EmptyState::is_empty)
    }

    #[inline]
    fn clear(&mut self) {
        for v in self {
            v.clear();
        }
    }
}

impl ForOverwrite for () {
    fn for_overwrite() -> Self {}
}

impl EmptyState for () {
    fn is_empty(&self) -> bool {
        true
    }

    fn clear(&mut self) {}
}
