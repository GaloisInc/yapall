// SPDX-License-Identifier: BSD-3-Clause
use std::fmt::Display;

use llvm_ir::{
    function::Parameter,
    module::{GlobalAlias, GlobalVariable},
    BasicBlock,
};

use crate::arc::UArc;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlobalName(String);

impl GlobalName {
    pub(crate) fn new(g: &GlobalVariable) -> Self {
        GlobalName(g.name.clone())
    }

    pub(crate) fn alias(g: &GlobalAlias) -> Self {
        GlobalName(g.name.clone())
    }

    pub(crate) fn _from_string(s: String) -> Self {
        GlobalName(s)
    }

    pub(crate) fn _from_str(s: &str) -> Self {
        GlobalName(s.to_string())
    }
}

impl From<&str> for GlobalName {
    fn from(s: &str) -> Self {
        GlobalName(s.to_string())
    }
}

impl Display for GlobalName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}", self.0)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FunctionName(String);

impl From<&str> for FunctionName {
    fn from(s: &str) -> Self {
        FunctionName(s.to_string())
    }
}

impl From<String> for FunctionName {
    fn from(s: String) -> Self {
        FunctionName(s)
    }
}

impl<T> PartialEq<T> for FunctionName
where
    T: AsRef<str>,
{
    fn eq(&self, other: &T) -> bool {
        self.0.as_str().eq(other.as_ref())
    }
}

impl FunctionName {
    pub(crate) fn definition(f: &llvm_ir::Function) -> Self {
        FunctionName(f.name.clone())
    }

    pub(crate) fn declaration(f: &llvm_ir::function::FunctionDeclaration) -> Self {
        FunctionName(f.name.clone())
    }

    pub(crate) fn contains(&self, s: &str) -> bool {
        self.0.contains(s)
    }

    pub(crate) fn starts_with(&self, s: &str) -> bool {
        self.0.starts_with(s)
    }

    pub(crate) fn get(&self) -> &str {
        &self.0
    }
}

impl Display for FunctionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}", self.0)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockName {
    parent_function: UArc<FunctionName>,
    name: llvm_ir::Name,
}

impl BlockName {
    // reduce heap allocations by precomputing string size
    fn name(&self) -> String {
        let bname = match &self.name {
            llvm_ir::Name::Name(n) => (**n).clone(),
            llvm_ir::Name::Number(n) => n.to_string(),
        };
        let mut s = String::with_capacity(
            self.parent_function.0.len()
            + 1 // :
            + bname.len(),
        );
        s += &self.parent_function.0;
        s += ":";
        s += &bname;
        s
    }

    pub(crate) fn new(parent_function: UArc<FunctionName>, b: &BasicBlock) -> Self {
        Self {
            parent_function,
            name: b.name.clone(),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstructionName {
    parent_function: UArc<FunctionName>,
    parent_block: UArc<BlockName>,
    idx: usize,
}

impl InstructionName {
    // reduce heap allocations by precomputing string size
    fn name(&self) -> String {
        let bname = self.parent_block.name();
        let iname = self.idx.to_string();
        let mut s = String::with_capacity(
            self.parent_function.0.len()
            + 1 // :
            + bname.len()
            + 1 // :
            + iname.len(), // idx; assume is usually < 999, i.e., length 3 when serialized
        );
        s += &self.parent_function.0;
        s += ":";
        s += &bname;
        s += ":";
        s += &iname;
        s
    }

    pub(crate) fn new(
        parent_function: UArc<FunctionName>,
        parent_block: UArc<BlockName>,
        idx: usize,
    ) -> Self {
        Self {
            parent_function,
            parent_block,
            idx,
        }
    }
}

impl Display for InstructionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ParameterName {
    parent_function: UArc<FunctionName>,
    name: llvm_ir::Name,
}

impl ParameterName {
    // reduce heap allocations by precomputing string size
    fn name(&self) -> String {
        let pname = match &self.name {
            llvm_ir::Name::Name(n) => (**n).clone(),
            llvm_ir::Name::Number(n) => n.to_string(),
        };
        let mut s = String::with_capacity(
            self.parent_function.0.len()
            + 1 // :
            + pname.len(),
        );
        s += &self.parent_function.0;
        s += ":";
        s += &pname;
        s
    }

    pub(crate) fn new(parent_function: UArc<FunctionName>, p: &Parameter) -> Self {
        Self {
            parent_function,
            name: p.name.clone(),
        }
    }
}

impl Display for ParameterName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum LocalName {
    Parameter(UArc<ParameterName>),
    Instruction(UArc<InstructionName>),
}

impl Display for LocalName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LocalName::Parameter(p) => format!("{}", p),
                LocalName::Instruction(i) => format!("{}", i),
            }
        )
    }
}
