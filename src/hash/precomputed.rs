// SPDX-License-Identifier: BSD-3-Clause
use std::hash::Hash;
use std::marker::PhantomData;

/// A precomputed hash for a value of type `T`.
///
/// The type parameter `T` is phantom, it exists solely to prevent mixing up
/// hashes of data of different types.
#[derive(Debug)]
pub(crate) struct PrecomputedHash<T> {
    pub(crate) hash: u64,
    pub(crate) phantom: PhantomData<T>,
}

/// Manually implemented to avoid unnecessary trait bound
impl<T> Copy for PrecomputedHash<T> {}

/// Manually implemented to avoid unnecessary trait bound
impl<T> Clone for PrecomputedHash<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

/// Manually implemented to avoid unnecessary trait bound
impl<T> PartialEq for PrecomputedHash<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.hash.eq(&other.hash)
    }
}

/// Manually implemented to avoid unnecessary trait bound
impl<T> Hash for PrecomputedHash<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

/// Manually implemented to avoid unnecessary trait bound
impl<T> Eq for PrecomputedHash<T> {}

/// Manually implemented to avoid unnecessary trait bound
impl<T> PartialOrd for PrecomputedHash<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.hash.cmp(&other.hash))
    }
}

/// Manually implemented to avoid unnecessary trait bound
impl<T> Ord for PrecomputedHash<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hash.cmp(&other.hash)
    }
}

impl<T> PrecomputedHash<T> {
    fn compute_hash(val: &T) -> u64
    where
        T: Hash,
    {
        use std::hash::Hasher;
        let mut hasher = rustc_hash::FxHasher::default();
        val.hash(&mut hasher);
        hasher.finish()
    }

    #[inline]
    pub(crate) fn unsafe_from_u64(hash: u64) -> Self {
        PrecomputedHash {
            hash,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn unsafe_coerce<R>(self) -> PrecomputedHash<R> {
        PrecomputedHash {
            hash: self.hash,
            phantom: PhantomData,
        }
    }

    pub(crate) fn new(val: &T) -> Self
    where
        T: Hash,
    {
        Self::unsafe_from_u64(Self::compute_hash(val))
    }

    /// Gets the precomputed hash, if available.
    #[inline]
    pub(crate) fn to_u64(self) -> u64 {
        self.hash
    }
}
