use std::fmt::Display;
use std::hash::Hash;
use std::ptr;

/// A unique read-only pointer to a `T`.
///
/// Uniqueness means that the programmer asserts that no two `RefHash<T>`s will
/// hold different references to equal `T`-values. Based on this assumption,
/// the [`Eq`] and [`Ord`] instances can simply compare the pointer values, and
/// [`Hash`] can just hash the pointer.
///
/// Compare to [`UArc`].
#[derive(Clone, Debug)]
pub struct RefHash<'a, T>(&'a T);

fn _assert_refhash_copy()
where
    RefHash<'static, Vec<()>>: Copy,
{
}

fn _assert_refhash_sync_send()
where
    RefHash<'static, ()>: Send + Sync,
{
}

impl<'a, T: Clone> Copy for RefHash<'a, T> {}

impl<'a, T> RefHash<'a, T> {
    pub fn new(ptr: &'a T) -> Self {
        Self(ptr)
    }

    pub fn _into_inner(self) -> &'a T {
        self.0
    }
}

impl<'a, T> RefHash<'a, &T> {
    pub fn _from_ref(&self) -> RefHash<'a, T> {
        RefHash(*self.0)
    }
}

/// Hashes the pointer
impl<'a, T> Hash for RefHash<'a, T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        ptr::hash(self.0, state)
    }
}

/// Compares the pointers
impl<'a, T> PartialEq for RefHash<'a, T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.0, other.0)
    }
}

/// Compares the pointers
impl<'a, T> Eq for RefHash<'a, T> {}

/// Only displays the value
impl<'a, T: Display + Hash> Display for RefHash<'a, T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a, T> AsRef<T> for RefHash<'a, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.0
    }
}

impl<'a, T> std::ops::Deref for RefHash<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}
