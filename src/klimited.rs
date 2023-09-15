// SPDX-License-Identifier:i BSD-3-Clause
// TODO: Specialize for 1-9 or so? `pushed` is where many heap allocations happen.
use std::collections::VecDeque;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct KLimited<T> {
    k: usize,
    elems: VecDeque<T>,
}

impl<T> KLimited<T> {
    pub fn new(k: usize, initial_elems: Vec<T>) -> Self {
        let mut elems = if k < 8 {
            VecDeque::with_capacity(k)
        } else {
            VecDeque::new()
        };
        elems.extend(initial_elems);
        KLimited { k, elems }
    }

    pub fn push(&mut self, t: T) {
        if self.elems.len() >= self.k {
            self.elems.pop_back();
        }
        if self.k > 0 {
            self.elems.push_front(t);
        }
    }

    pub fn pushed(&self, t: T) -> Self
    where
        T: Clone,
    {
        let mut new = self.clone();
        new.push(t);
        new
    }

    // TODO
    #[allow(clippy::should_implement_trait)]
    pub fn into_iter(self) -> impl Iterator<Item = T> {
        self.elems.into_iter()
    }
}
