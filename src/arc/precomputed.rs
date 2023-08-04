use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::ops::Deref;

use crate::hash::PreHashed;

/// A thread-safe reference-counting pointer like [`std::sync::Arc<T>`] that
/// caches the hash of its contained value, ensuring it will be calculated
/// exactly once.
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Arc<T>(triomphe::Arc<PreHashed<T>>);

fn _assert_arc_sync_send()
where
    Arc<()>: Send + Sync,
{
}

fn _assert_arc_clone()
where
    Arc<std::sync::Mutex<()>>: Clone,
{
}

impl<T> Arc<T> {
    pub fn new(val: T) -> Self
    where
        T: Hash,
    {
        Self(triomphe::Arc::new(PreHashed::new(val)))
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().as_ref()
    }
}

impl<T: Display + Hash> Display for Arc<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> AsRef<T> for Arc<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.0.as_ref().as_ref()
    }
}
