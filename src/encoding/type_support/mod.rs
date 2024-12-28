#[cfg(feature = "arrayvec")]
mod arrayvec;
#[cfg(feature = "bstr")]
mod bstr;
#[cfg(feature = "bytestring")]
mod bytestring;
#[cfg(feature = "chrono")]
mod chrono;
#[cfg(feature = "hashbrown")]
mod hashbrown;

#[cfg(feature = "smallvec")]
mod impl_smallvec {
    use crate::encoding::{delegate_encoding, General, Unpacked};

    delegate_encoding!(delegate from (General) to (Unpacked<General>)
        for type (smallvec::SmallVec<A>) including distinguished
        with where clause (A: smallvec::Array<Item = T>)
        with generics (T, A));
}

#[cfg(feature = "std")]
mod impl_std {
    use crate::encoding::{delegate_encoding, delegate_value_encoding, General, Map, Unpacked};

    delegate_encoding!(delegate from (General) to (Unpacked<General>)
        for type (std::collections::HashSet<T, S>)
        with where clause (S: Default + core::hash::BuildHasher)
        with generics (T, S));
    delegate_value_encoding!(delegate from (General) to (Map<General, General>)
        for type (std::collections::HashMap<K, V, S>)
        with where clause (K: Eq + core::hash::Hash, S: Default + core::hash::BuildHasher)
        with generics (K, V, S));
}

#[cfg(feature = "thin-vec")]
mod impl_thin_vec {
    use crate::encoding::{delegate_encoding, General, Unpacked};

    delegate_encoding!(delegate from (General) to (Unpacked<General>)
        for type (thin_vec::ThinVec<T>) including distinguished with generics (T));
}

#[cfg(feature = "tinyvec")]
mod impl_tinyvec {
    use crate::encoding::{delegate_encoding, General, Unpacked};

    delegate_encoding!(delegate from (General) to (Unpacked<General>)
        for type (tinyvec::ArrayVec<A>) including distinguished
        with where clause (A: tinyvec::Array<Item = T>)
        with generics (T, A));
    delegate_encoding!(delegate from (General) to (Unpacked<General>)
        for type (tinyvec::TinyVec<A>) including distinguished
        with where clause (A: tinyvec::Array<Item = T>)
        with generics (T, A));
}
