// SPDX-License-Identifier: BSD-3-Clause
use std::fmt::Display;
use std::hash::Hash;
use std::ptr;

use triomphe::Arc;

/// A thread-safe, reference-counting pointer to a unique `T`.
///
/// Uniqueness means that the programmer asserts that no two `UArc<T>`s will
/// hold equal `T`-values. Based on this assumption, the [`Eq`] and [`Ord`]
/// instances can simply compare the pointer values, and [`Hash`] can just hash
/// the pointer.
///
/// Compare to [`RefHash`].
#[derive(Debug)]
pub struct UArc<T>(Arc<T>);

fn _assert_uarc_clone()
where
    UArc<std::sync::Mutex<()>>: Clone,
{
}

fn _assert_prehashed_sync_send()
where
    UArc<&'static mut ()>: Send + Sync,
{
}

impl<T> UArc<T> {
    pub(crate) fn new(t: T) -> Self {
        Self(Arc::new(t))
    }

    pub fn into_arc(self) -> Arc<T> {
        self.0
    }
}

impl<T> Clone for UArc<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// Hashes the pointer
impl<T> Hash for UArc<T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        ptr::hash(Arc::as_ptr(&self.0), state)
    }
}

/// Compares the pointers
impl<T> PartialEq for UArc<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(Arc::as_ptr(&self.0), Arc::as_ptr(&other.0))
    }
}

/// Compares the pointers
impl<T> Eq for UArc<T> {}

/// Compares the pointers
impl<T> PartialOrd for UArc<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Compares the pointers
impl<T> Ord for UArc<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (Arc::as_ptr(&self.0) as usize).cmp(&(Arc::as_ptr(&other.0) as usize))
    }
}

/// Only displays the value
impl<T: Display + Hash> Display for UArc<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> AsRef<T> for UArc<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

impl<T> std::ops::Deref for UArc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
