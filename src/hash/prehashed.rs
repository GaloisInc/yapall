// SPDX-License-Identifier: BSD-3-Clause
use std::fmt::{Debug, Display};
use std::hash::Hash;

use super::PrecomputedHash;

/// A wrapper for `T` that precomputes its hash.
#[derive(Clone, Copy, Debug)]
pub struct PreHashed<T> {
    val: T,
    hash: PrecomputedHash<T>,
}

fn _assert_prehashed_copy()
where
    PreHashed<&'static ()>: Copy,
{
}

fn _assert_prehashed_sync_send()
where
    PreHashed<()>: Send + Sync,
{
}

impl<T> PreHashed<T> {
    pub(crate) fn new(val: T) -> Self
    where
        T: Hash,
    {
        let hash = PrecomputedHash::new(&val);
        Self { val, hash }
    }

    pub(crate) fn for_ref(&self) -> PreHashed<&T> {
        PreHashed {
            val: &self.val,
            hash: self.hash.unsafe_coerce(),
        }
    }

    pub fn into_inner(self) -> T {
        self.val
    }

    /// The function shouldn't change the hash of the value
    pub(crate) fn _unsafe_map<R, F: FnOnce(T) -> R>(self, f: F) -> PreHashed<R> {
        PreHashed {
            val: f(self.val),
            hash: self.hash.unsafe_coerce(),
        }
    }
}

impl<T: Clone> PreHashed<&T> {
    fn _from_ref(&self) -> PreHashed<T> {
        PreHashed {
            val: self.val.clone(),
            hash: PrecomputedHash::unsafe_from_u64(self.hash.to_u64()),
        }
    }
}

impl<T: Hash> Hash for PreHashed<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash.to_u64());
    }
}

/// Compares the hashes
impl<T> PartialEq for PreHashed<T> {
    fn eq(&self, other: &Self) -> bool {
        self.hash.eq(&other.hash)
    }
}

/// Compares the hashes
impl<T> Eq for PreHashed<T> {}

/// Compares the hashes
impl<T> PartialOrd for PreHashed<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.hash.cmp(&other.hash))
    }
}

/// Compares the hashes
impl<T> Ord for PreHashed<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hash.cmp(&other.hash)
    }
}

/// Only displays the value
impl<T: Display + Hash> Display for PreHashed<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.val.fmt(f)
    }
}

impl<T> AsRef<T> for PreHashed<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.val
    }
}
