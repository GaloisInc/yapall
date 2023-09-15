// SPDX-License-Identifier:i BSD-3-Clause
//! Lattice for values of integer variables

use ascent::{lattice::constant_propagation::ConstPropagation, Lattice};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Int {
    pub bits: u32,
    pub value: u64,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct IntLattice(ConstPropagation<Int>);

// ------------------------------------------------------------------
// Constructors

impl IntLattice {
    #[inline]
    pub fn bottom() -> Self {
        IntLattice(ConstPropagation::Bottom)
    }

    #[inline]
    pub fn constant(i: Int) -> Self {
        IntLattice(ConstPropagation::Constant(i))
    }

    #[inline]
    pub fn top() -> Self {
        IntLattice(ConstPropagation::Top)
    }
}

// ------------------------------------------------------------------
// Operations

impl IntLattice {
    #[inline]
    pub fn add(&self, _other: &Self) -> Self {
        // TODO: Signed + unsigned wrap?
        Self::top()
        // IntLattice(match (self.0, other.0) {
        //     (ConstPropagation::Bottom, ConstPropagation::Bottom) => ConstPropagation::Bottom,
        //     (ConstPropagation::Bottom, c @ ConstPropagation::Constant(_)) => c,
        //     (ConstPropagation::Bottom, ConstPropagation::Top) => ConstPropagation::Top,
        //     (c @ ConstPropagation::Constant(_), ConstPropagation::Bottom) => c,
        //     (ConstPropagation::Constant(_), ConstPropagation::Top) => ConstPropagation::Top,
        //     (ConstPropagation::Top, ConstPropagation::Bottom) => ConstPropagation::Top,
        //     (ConstPropagation::Top, ConstPropagation::Constant(_)) => ConstPropagation::Top,
        //     (ConstPropagation::Top, ConstPropagation::Top) => ConstPropagation::Top,
        //     (ConstPropagation::Constant(c), ConstPropagation::Constant(d)) => {
        //         if c.bits == d.bits {
        //             ConstPropagation::Constant(Int {
        //                 bits: c.bits,
        //                 value: c.value + d.value,
        //             })
        //         } else {
        //             ConstPropagation::Top
        //         }
        //     }
        // })
    }

    #[inline]
    pub fn div(&self, _other: &Self) -> Self {
        Self::top()
    }

    #[inline]
    pub fn mul(&self, _other: &Self) -> Self {
        Self::top()
    }

    #[inline]
    pub fn sub(&self, _other: &Self) -> Self {
        Self::top()
    }
}

// ------------------------------------------------------------------
// Traits

impl std::fmt::Display for IntLattice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            ConstPropagation::Bottom => write!(f, "⊥"),
            ConstPropagation::Constant(c) => write!(f, "{}: i{}", c.value, c.bits),
            ConstPropagation::Top => write!(f, "⊤"),
        }
    }
}

impl Lattice for IntLattice {
    fn meet(self, other: Self) -> Self {
        IntLattice(self.0.meet(other.0))
    }

    fn join(self, other: Self) -> Self {
        IntLattice(self.0.join(other.0))
    }
}
