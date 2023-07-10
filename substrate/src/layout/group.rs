//! Groups of layout objects.

/// A group of one or more layout objects.
pub struct Group<T> {
    inner: T,
}

impl<T> Group<T> {
    /// Creates a new group containing the given object(s).
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Returns a reference to the contents of the group.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Consumes the group, returning its contents.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> From<T> for Group<T> {
    fn from(value: T) -> Self {
        Self { inner: value }
    }
}
