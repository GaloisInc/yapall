// SPDX-License-Identifier: BSD-3-Clause
use std::ops::{Deref, DerefMut};

use crate::arc::Arc;

pub trait UnionFind: Sized {
    type Ref<'a>: Deref<Target = Option<Arc<Self>>>
    where
        Self: 'a;
    type MutRef<'a>: DerefMut<Target = Option<Arc<Self>>>
    where
        Self: 'a;

    fn parent_ref(&self) -> Self::Ref<'_>;

    fn parent_mut_ref(&self) -> Self::MutRef<'_>;

    fn grandparent(a: &Arc<Self>) -> Option<(Arc<Self>, Arc<Self>)>
    where
        Self: PartialEq,
    {
        match &*a.parent_ref() {
            None => None,
            Some(p) => {
                debug_assert!(a != p);
                match &*p.parent_ref() {
                    None => None,
                    Some(gp) => {
                        debug_assert!(a != gp);
                        debug_assert!(p != gp);
                        Some((p.clone(), gp.clone()))
                    }
                }
            }
        }
    }

    fn lookup(a: &Arc<Self>) -> Arc<Self>
    where
        Self: PartialEq,
    {
        let last = {
            let mut current = a.clone();
            while let Some((p, gp)) = Self::grandparent(&current) {
                {
                    *a.parent_mut_ref() = Some(gp.clone());
                    current = p;
                }
            }
            current
        };
        let bind = match &*last.parent_ref() {
            None => last.clone(),
            Some(p) => {
                debug_assert!(a != p);
                debug_assert!(last != *p);
                p.clone()
            }
        };
        bind
    }

    // TODO: Merge by weight
    fn merge(a: &Arc<Self>, b: &Arc<Self>) -> bool
    where
        Self: PartialEq + PartialOrd,
    {
        if a == b {
            return false;
        }
        let ar = Self::lookup(a);
        let br = Self::lookup(b);
        if ar == br {
            return false;
        }
        let (less, greater) = if ar < br { (ar, br) } else { (br, ar) };
        debug_assert!(less != greater);
        {
            let mut parent = greater.parent_mut_ref();
            *parent = Some(less);
        }
        true
    }
}
