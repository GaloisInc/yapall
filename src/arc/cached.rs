use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::ops::Deref;

use crate::cached_hash::CachedHash;

/// A thread-safe reference-counting pointer like [`std::sync::Arc<T>`] that
/// caches the hash of its contained value (in a [`CachedHash`]), ensuring it
/// will be calculated (at most) once.
#[derive(Clone, Debug)]
pub struct Arc<T> {
    rc: std::sync::Arc<T>,
    hash: CachedHash<T>,
}

fn _assert_arc_sync_send()
where
    Arc<()>: Send + Sync,
{
}

impl<T> Arc<T> {
    pub fn new(x: T) -> Self {
        Self {
            rc: std::sync::Arc::new(x),
            hash: Default::default(),
        }
    }
}

impl<T: Hash> Hash for Arc<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash.get_or_init(&self).get());
    }
}

impl<T: PartialEq> PartialEq for Arc<T> {
    fn eq(&self, other: &Self) -> bool {
        self.rc == other.rc
    }
}

impl<T: Eq> Eq for Arc<T> {}

impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.rc.as_ref()
    }
}

impl<T: PartialOrd> PartialOrd for Arc<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.rc.partial_cmp(&other.rc)
    }
}

impl<T: Ord> Ord for Arc<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rc.cmp(&other.rc)
    }
}

impl<T: Display + Hash> Display for Arc<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.rc.fmt(f)
    }
}

impl<T> AsRef<T> for Arc<T> {
    fn as_ref(&self) -> &T {
        self.rc.as_ref()
    }
}
