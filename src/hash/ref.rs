// SPDX-License-Identifier: BSD-3-Clause
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
pub(crate) struct RefHash<'a, T>(&'a T);

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

impl<T: Clone> Copy for RefHash<'_, T> {}

impl<'a, T> RefHash<'a, T> {
    pub(crate) fn new(ptr: &'a T) -> Self {
        Self(ptr)
    }

    fn _into_inner(self) -> &'a T {
        self.0
    }
}

impl<T> RefHash<'_, &T> {
    fn _from_ref(&self) -> RefHash<'_, T> {
        RefHash(*self.0)
    }
}

/// Hashes the pointer
impl<T> Hash for RefHash<'_, T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        ptr::hash(self.0, state)
    }
}

/// Compares the pointers
impl<T> PartialEq for RefHash<'_, T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.0, other.0)
    }
}

/// Compares the pointers
impl<T> Eq for RefHash<'_, T> {}

/// Only displays the value
impl<T: Display + Hash> Display for RefHash<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> AsRef<T> for RefHash<'_, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.0
    }
}

impl<T> std::ops::Deref for RefHash<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}
