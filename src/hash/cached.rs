// SPDX-License-Identifier:i BSD-3-Clause
// TODO(lb): Add LICENSE from OnceCell

#![allow(unused)]

use std::hash::Hash;
use std::marker::PhantomData;
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};

/// A cached hash of a value of type `T`.
///
/// Implemented as an [`AtomicU64`], with `0` representing "not yet
/// computed".
///
/// The type parameter `T` is phantom, it exists solely to prevent mixing up
/// hashes of data of different types.
///
/// Like once_cell's `OnceNonZeroU64`, except that:
///
/// - It's a `u64`, which matches Rust's [`Hash`] API.
/// - It has a phantom type parameter `T` for extra type safety.
/// - It can only be set by actually hashing a value.
#[derive(Debug)]
pub(crate) struct CachedHash<T> {
    hash: AtomicU64,
    phantom: PhantomData<T>,
}

impl<T> CachedHash<T> {
    fn compute_hash(val: &T) -> NonZeroU64
    where
        T: Hash,
    {
        use std::hash::Hasher;
        let mut hasher = rustc_hash::FxHasher::default();
        val.hash(&mut hasher);
        let mut hash = hasher.finish();
        if hash == 0 {
            hash += 1;
        }
        unsafe { NonZeroU64::new_unchecked(hash) }
    }

    fn from_u64(hash: u64) -> Self {
        CachedHash {
            hash: AtomicU64::new(hash),
            phantom: PhantomData,
        }
    }

    fn from_non_zero(hash: NonZeroU64) -> Self {
        Self::from_u64(hash.get())
    }

    /// Create a new [`CachedHash`] by immediately hashing the value.
    pub(crate) fn cached(val: &T) -> Self
    where
        T: Hash,
    {
        Self::from_non_zero(Self::compute_hash(val))
    }

    /// Gets the cached hash, if available.
    #[inline]
    pub(crate) fn get(&self) -> Option<NonZeroU64> {
        let val = self.hash.load(Ordering::Acquire);
        NonZeroU64::new(val)
    }

    /// Retrieve the cached hash if available, or compute and cache it.
    ///
    /// Implementation inspired by
    /// `once_cell::race::OnceNonZeroUsize::get_or_init`.
    pub(crate) fn get_or_init(&self, val: &T) -> NonZeroU64
    where
        T: Hash,
    {
        let maybe_hash = self.hash.load(Ordering::Acquire);
        match NonZeroU64::new(maybe_hash) {
            Some(it) => it,
            None => {
                let mut hash = Self::compute_hash(val).get();
                let exchange =
                    self.hash
                        .compare_exchange(0, hash, Ordering::AcqRel, Ordering::Acquire);
                if let Err(old) = exchange {
                    hash = old;
                }
                unsafe { NonZeroU64::new_unchecked(hash) }
            }
        }
    }
}

/// Default: unset (i.e., zero)
impl<T> Default for CachedHash<T> {
    fn default() -> Self {
        Self::from_u64(0)
    }
}

impl<T> Clone for CachedHash<T> {
    fn clone(&self) -> Self {
        Self::from_u64(self.hash.load(Ordering::SeqCst))
    }
}

#[cfg(test)]
mod tests {
    use super::CachedHash;

    #[test]
    fn default_get() {
        assert!(CachedHash::<u8>::default().get().is_none());
    }

    #[test]
    fn cached_get() {
        assert!(CachedHash::<u8>::cached(&0).get().is_some());
        assert!(CachedHash::<u8>::cached(&u8::MAX).get().is_some());
        assert!(CachedHash::<u64>::cached(&0).get().is_some());
        assert!(CachedHash::<u64>::cached(&u64::MAX).get().is_some());
    }

    #[test]
    fn get_or_init_twice() {
        let cached = CachedHash::<u8>::default();
        assert_eq!(cached.get_or_init(&0), cached.get_or_init(&0));
    }

    #[test]
    fn get_or_init_neq() {
        let cached = CachedHash::<u8>::default();
        assert!(cached.get_or_init(&0) == cached.get_or_init(&1));
    }
}
