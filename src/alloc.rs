// SPDX-License-Identifier: BSD-3-Clause
// TODO: Heap cloning, with two optimizations:
//
// - Stack allocations with non-pointer-containing types need not have contexts
// - Heap allocations smaller than a pointer need not have contexts

use std::{fmt::Display, sync::RwLock};

use triomphe::Arc as SArc;

use crate::{
    arc::{Arc, UArc},
    llvm::instruction::Alloca,
    llvm::{FunctionName, GlobalName, InstructionName},
    union::UnionFind,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FunctionAlloc(UArc<FunctionName>);

fn _assert_function_alloc_send_sync()
where
    FunctionAlloc: Send + Sync,
{
}

impl FunctionAlloc {
    #[inline]
    pub fn new(f: UArc<FunctionName>) -> Self {
        FunctionAlloc(f)
    }

    #[inline]
    pub fn function_name(&self) -> UArc<FunctionName> {
        self.0.clone()
    }
}

impl Display for FunctionAlloc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "*{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct GlobalAlloc {
    global: Arc<GlobalName>,
    pub(crate) constant: bool,
    pub(crate) size: Option<u64>,
    // TODO: Cargo feature to disable this field
    parent: SArc<RwLock<Option<Arc<GlobalAlloc>>>>,
}

/// Only hashes the underlying global name, since only one [`GlobalAlloc`] is
/// created per [`GlobalName`].
impl std::hash::Hash for GlobalAlloc {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.global.hash(state);
    }
}

/// Only compares the underlying global name, since only one [`GlobalAlloc`] is
/// created per [`GlobalName`].
impl PartialEq for GlobalAlloc {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.global == other.global
    }
}

/// See comment on [`PartialEq`].
impl Eq for GlobalAlloc {}

/// See comment on [`PartialEq`].
impl PartialOrd for GlobalAlloc {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.global.cmp(&other.global))
    }
}

/// See comment on [`PartialEq`].
impl Ord for GlobalAlloc {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.global.cmp(&other.global)
    }
}

impl GlobalAlloc {
    pub fn new(global: Arc<GlobalName>, constant: bool, size: Option<u64>) -> Arc<Self> {
        Arc::new(GlobalAlloc {
            global,
            constant,
            size,
            parent: SArc::new(RwLock::new(None)),
        })
    }

    fn do_merge(a: &Arc<Self>, b: &Arc<Self>) -> bool {
        if a.constant == b.constant {
            Self::merge(a, b)
        } else {
            false
        }
    }
}

impl UnionFind for GlobalAlloc {
    type Ref<'a> = std::sync::RwLockReadGuard<'a, Option<Arc<Self>>>;

    type MutRef<'a> = std::sync::RwLockWriteGuard<'a, Option<Arc<Self>>>;

    #[inline]
    fn parent_ref(&self) -> Self::Ref<'_> {
        self.parent.read().unwrap()
    }

    #[inline]
    fn parent_mut_ref(&self) -> Self::MutRef<'_> {
        self.parent.write().unwrap()
    }
}

impl Display for GlobalAlloc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "*{}:{}({})",
            self.global,
            self.constant,
            self.size
                .map(|s| s.to_string())
                .unwrap_or_else(|| "_".to_string())
        )
    }
}

#[derive(Clone, Debug)]
pub struct HeapAlloc {
    // TODO: Include the allocation function/signature
    instruction: UArc<InstructionName>,
    size: Option<u64>,
    parent: SArc<RwLock<Option<Arc<HeapAlloc>>>>,
}

/// Only hashes the underlying instruction name, since only one [`HeapAlloc`]
/// is created per allocating instruction.
impl std::hash::Hash for HeapAlloc {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.instruction.hash(state);
    }
}

/// Only compares the underlying instruction name, since only one [`HeapAlloc`]
/// is created per allocating instruction.
impl PartialEq for HeapAlloc {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.instruction == other.instruction
    }
}

/// See comment on [`PartialEq`].
impl Eq for HeapAlloc {}

impl PartialOrd for HeapAlloc {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.instruction.cmp(&other.instruction))
    }
}

/// See comment on [`PartialEq`].
impl Ord for HeapAlloc {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.instruction.cmp(&other.instruction)
    }
}

impl UnionFind for HeapAlloc {
    type Ref<'a> = std::sync::RwLockReadGuard<'a, Option<Arc<Self>>>;

    type MutRef<'a> = std::sync::RwLockWriteGuard<'a, Option<Arc<Self>>>;

    #[inline]
    fn parent_ref(&self) -> Self::Ref<'_> {
        self.parent.read().unwrap()
    }

    #[inline]
    fn parent_mut_ref(&self) -> Self::MutRef<'_> {
        self.parent.write().unwrap()
    }
}

impl HeapAlloc {
    pub fn new(instruction: UArc<InstructionName>, size: Option<u64>) -> Arc<Self> {
        Arc::new(HeapAlloc {
            instruction,
            size,
            parent: SArc::new(RwLock::new(None)),
        })
    }
}

impl Display for HeapAlloc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "*{}({})",
            self.instruction,
            self.size
                .map(|s| s.to_string())
                .unwrap_or_else(|| "_".to_string())
        )
    }
}

#[derive(Clone, Debug)]
pub struct StackAlloc {
    // TODO: Include the allocation function/signature
    name: UArc<InstructionName>,
    parent: SArc<RwLock<Option<Arc<StackAlloc>>>>,
}

/// Only hashes the underlying instruction name, since only one [`StackAlloc`]
/// is created per allocating instruction.
impl std::hash::Hash for StackAlloc {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

/// Only compares the underlying instruction name, since only one [`StackAlloc`]
/// is created per allocating instruction.
impl PartialEq for StackAlloc {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for StackAlloc {}

impl PartialOrd for StackAlloc {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

impl Ord for StackAlloc {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl UnionFind for StackAlloc {
    type Ref<'a> = std::sync::RwLockReadGuard<'a, Option<Arc<Self>>>;

    type MutRef<'a> = std::sync::RwLockWriteGuard<'a, Option<Arc<Self>>>;

    #[inline]
    fn parent_ref(&self) -> Self::Ref<'_> {
        self.parent.read().unwrap()
    }

    #[inline]
    fn parent_mut_ref(&self) -> Self::MutRef<'_> {
        self.parent.write().unwrap()
    }
}

impl StackAlloc {
    pub fn alloca(name: UArc<InstructionName>, _a: &Alloca) -> Arc<Self> {
        Arc::new(StackAlloc {
            name,
            parent: SArc::new(RwLock::new(None)),
        })
    }

    pub fn signature(name: UArc<InstructionName>) -> Arc<Self> {
        Arc::new(StackAlloc {
            name,
            parent: SArc::new(RwLock::new(None)),
        })
    }
}

impl Display for StackAlloc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "*{}", self.name,)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Alloc {
    Function(FunctionAlloc),
    Global(Arc<GlobalAlloc>),
    Heap(Arc<HeapAlloc>),
    Stack(Arc<StackAlloc>),
    //
    Null,
    Top,
}

impl Alloc {
    pub(crate) fn lookup(a: &Arc<Self>) -> Arc<Self> {
        match &**a {
            Alloc::Stack(b) => {
                let root = StackAlloc::lookup(b);
                if root == *b {
                    return a.clone();
                }
                Arc::new(Alloc::Stack(root))
            }
            _ => a.clone(),
        }
    }

    pub(crate) fn merge(&self, o: &Alloc) -> bool {
        if self == o {
            return false;
        }
        match (&self, o) {
            (Alloc::Global(p), Alloc::Global(q)) => GlobalAlloc::do_merge(p, q),
            (Alloc::Heap(p), Alloc::Heap(q)) if p.size == q.size => HeapAlloc::merge(p, q),
            (Alloc::Stack(p), Alloc::Stack(q)) => StackAlloc::merge(p, q),
            _ => false,
        }
    }

    pub(crate) fn freeable(&self) -> bool {
        match self {
            Alloc::Heap(_) => true,
            Alloc::Top => true,
            // No `_` pattern to ensure this is updated if the type changes
            Alloc::Function(_) => false,
            Alloc::Global(_) => false,
            Alloc::Null => false,
            Alloc::Stack(_) => false,
        }
    }

    pub(crate) fn loadable(&self) -> bool {
        match self {
            Alloc::Function(_) => false,
            Alloc::Null => false,
            // No `_` pattern to ensure this is updated if the type changes
            Alloc::Global(_) => true,
            Alloc::Heap(_) => true,
            Alloc::Stack(_) => true,
            Alloc::Top => true,
        }
    }

    pub(crate) fn storable(&self) -> bool {
        match self {
            Alloc::Function(_) => false,
            Alloc::Global(g) => !g.constant,
            Alloc::Null => false,
            // No `_` pattern to ensure this is updated if the type changes
            Alloc::Heap(_) => true,
            Alloc::Stack(_) => true,
            Alloc::Top => true,
        }
    }
}

impl Display for Alloc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Alloc::Function(f) => format!("{}", f),
                Alloc::Global(g) => format!("{}", g),
                Alloc::Heap(h) => format!("{}", h),
                Alloc::Stack(s) => format!("{}", s),
                //
                Alloc::Null => "*null".to_string(),
                Alloc::Top => "Top".to_string(),
            }
        )
    }
}

// TODO: fails!
// #[cfg(test)]
// mod tests {
//     use super::StackAlloc;
//     use crate::arc::Arc;
//     use crate::union::UnionFind;

//     #[test]
//     fn stack_merge() {
//         let a = Arc::new(StackAlloc::from("a"));
//         assert_eq!(a, StackAlloc::lookup(&a));
//         let b = Arc::new(StackAlloc::from("b"));
//         assert_eq!(b, StackAlloc::lookup(&b));

//         StackAlloc::merge(&a, &b);
//         assert_eq!(a, StackAlloc::lookup(&a));
//         assert!(b != StackAlloc::lookup(&b));
//         assert_eq!(StackAlloc::lookup(&a), StackAlloc::lookup(&b));
//     }
// }
