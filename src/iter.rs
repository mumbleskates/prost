//! Iterator adapters used by the crate that are not available elsewhere.

/// Adapter that allows flattening an iterator of (K: Clone, V: IntoIter) into (K, V::Item).
/// This is useful as where the type of core::iter::FlatMap cannot be named (because its function
/// type is always anonymous), the type of FlatAdapter(..).flatten() can be named any time the type
/// of its iterator can.
pub struct FlatAdapter<I>(pub I);

impl<I, K, Vs> Iterator for FlatAdapter<I>
where
    I: Iterator<Item = (K, Vs)> + Sized,
    K: Clone,
    Vs: IntoIterator,
{
    type Item = Flattening<K, Vs::IntoIter>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, v)| Flattening(k, v.into_iter()))
    }
}

impl<I, K, Vs> ExactSizeIterator for FlatAdapter<I>
where
    I: ExactSizeIterator<Item = (K, Vs)> + Sized,
    K: Clone,
    Vs: IntoIterator,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<I, K, Vs> DoubleEndedIterator for FlatAdapter<I>
where
    I: DoubleEndedIterator<Item = (K, Vs)> + Sized,
    K: Clone,
    Vs: IntoIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0
            .next_back()
            .map(|(k, v)| Flattening(k, v.into_iter()))
    }
}

/// Iterator for an individual (K: Clone, V: Iterator) that produces (K, V::Item).
pub struct Flattening<K, Vi>(K, Vi);

impl<K, Vi> Iterator for Flattening<K, Vi>
where
    K: Clone,
    Vi: Iterator,
{
    type Item = (K, Vi::Item);

    fn next(&mut self) -> Option<Self::Item> {
        self.1.next().map(|v| (self.0.clone(), v))
    }
}

impl<K, Vi> ExactSizeIterator for Flattening<K, Vi>
where
    K: Clone,
    Vi: ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.1.len()
    }
}

impl<K, Vi> DoubleEndedIterator for Flattening<K, Vi>
where
    K: Clone,
    Vi: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.1.next_back().map(|v| (self.0.clone(), v))
    }
}
