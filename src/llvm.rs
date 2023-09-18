// SPDX-License-Identifier: BSD-3-Clause
//! Representation of an LLVM module that is amenable to analysis. In
//! particular, Ascent values must implement `Clone`, `Hash`, and `Eq`. Since
//! the LLVM AST contains floats, and floats don't implement `Eq`, we must
//! reproduce large parts of the LLVM module structure.
//!
//! In addition to the trait issue, redefining the structure of an LLVM module
//! also allows us to make a few tweaks that make the structure of the analysis
//! itself a bit simpler and performant, such as wrapping things in [`Arc`].

use std::collections::HashMap;

use llvm_ir::{types::Typed, Name};

use crate::arc::{Arc, UArc};
use crate::hash::PreHashed;

use self::constant::Constant;
use self::instruction::Opcode;

pub mod constant;
mod error;
pub use error::*;
mod name;
pub use name::*;
mod operand;
pub use operand::*;
pub mod instruction;
pub mod terminator;
pub use terminator::*;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Instruction {
    pub(crate) name: UArc<InstructionName>,
    pub opcode: PreHashed<Opcode>,
    pub(crate) ty: llvm_ir::TypeRef,
}

impl Instruction {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new<'module>(
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        instructions: &HashMap<&Name, UArc<InstructionName>>,
        m: &'module llvm_ir::Module,
        i: &'module llvm_ir::Instruction,
        f_name: &UArc<FunctionName>,
        b_name: &UArc<BlockName>,
        idx: usize,
    ) -> Result<Self, Error> {
        Ok(Instruction {
            name: {
                if let Some(n) = i.try_get_result() {
                    instructions.get(n).unwrap().clone()
                } else {
                    // NOTE! It is important that this is one of three sites
                    // where `UArc::new(InstructionName)` is called (and these
                    // callsites don't create the same `InstructionName`s),
                    // see docs for `UArc`.
                    UArc::new(InstructionName::new(f_name.clone(), b_name.clone(), idx))
                }
            },
            opcode: PreHashed::new(Opcode::from_instruction(
                &m.data_layout,
                &m.types,
                operands,
                globals,
                locals,
                i,
            )?),
            ty: i.get_type(&m.types),
        })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Terminator {
    pub(crate) name: UArc<InstructionName>,
    pub opcode: PreHashed<TerminatorOpcode>,
    pub(crate) ty: llvm_ir::TypeRef,
}

impl Terminator {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new<'module>(
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        instructions: &HashMap<&Name, UArc<InstructionName>>,
        m: &'module llvm_ir::Module,
        b: &'module llvm_ir::BasicBlock,
        i: &'module llvm_ir::Terminator,
        f_name: &UArc<FunctionName>,
        b_name: &UArc<BlockName>,
    ) -> Result<Self, Error> {
        Ok(Terminator {
            name: {
                if let Some(n) = i.try_get_result() {
                    instructions.get(n).unwrap().clone()
                } else {
                    UArc::new(InstructionName::new(
                        f_name.clone(),
                        b_name.clone(),
                        b.instrs.len() + 1,
                    ))
                }
            },
            opcode: PreHashed::new(TerminatorOpcode::from_terminator(
                operands, globals, locals, i,
            )?),
            ty: i.get_type(&m.types),
        })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Block {
    pub(crate) name: UArc<BlockName>,
    pub instrs: Vec<Instruction>,
    pub terminator: Arc<Terminator>,
}

impl Block {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new<'module>(
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        instructions: &HashMap<&Name, UArc<InstructionName>>,
        m: &'module llvm_ir::Module,
        b: &'module llvm_ir::BasicBlock,
        f_name: &UArc<FunctionName>,
        name: UArc<BlockName>,
    ) -> Result<Self, Error> {
        let mut instrs = Vec::with_capacity(b.instrs.len());
        for (idx, i) in b.instrs.iter().enumerate() {
            instrs.push(Instruction::new(
                operands,
                globals,
                locals,
                instructions,
                m,
                i,
                f_name,
                &name,
                idx,
            )?);
        }
        let terminator = Arc::new(Terminator::new(
            operands,
            globals,
            locals,
            instructions,
            m,
            b,
            &b.term,
            f_name,
            &name,
        )?);
        Ok(Block {
            name,
            instrs,
            terminator,
        })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Function {
    pub parameters: Vec<Arc<Operand>>,
    pub blocks: Vec<Block>,
    pub return_type: llvm_ir::TypeRef,
}

impl Function {
    pub(crate) fn new<'module>(
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        m: &'module llvm_ir::Module,
        f: &'module llvm_ir::Function,
        name: &UArc<FunctionName>,
    ) -> Result<Self, Error> {
        let mut instructions = HashMap::<&Name, UArc<InstructionName>>::new();
        let mut locals = HashMap::<&Name, Arc<Operand>>::new();

        let mut parameters = Vec::with_capacity(f.parameters.len());
        for p in &f.parameters {
            // NOTE! It is important that this the only site where
            // `UArc::new(ParameterName)` is called, see docs for `UArc`.
            let p_name = UArc::new(ParameterName::new(name.clone(), p));
            let op = Arc::new(Operand::Local(Arc::new(LocalName::Parameter(p_name))));
            parameters.push(op.clone());
            locals.insert(&p.name, op);
        }

        // Have to save block names from the below loop to preserve UArc
        // invariant.
        let mut block_names = Vec::with_capacity(f.basic_blocks.len());

        for b in &f.basic_blocks {
            let block_name = UArc::new(BlockName::new(name.clone(), b));

            for (idx, instr) in b.instrs.iter().enumerate() {
                if let Some(n) = instr.try_get_result() {
                    // NOTE! It is important that this is one of three sites where
                    // `UArc::new(InstructionName)` is called (and these callsites
                    // don't create the same `InstructionName`s), see docs for
                    // `UArc`.
                    let inst_name =
                        UArc::new(InstructionName::new(name.clone(), block_name.clone(), idx));
                    instructions.insert(n, inst_name.clone());

                    // NOTE[pass-thru] TODO: See #32.
                    let get_local = |op: &llvm_ir::Operand| match op {
                        llvm_ir::Operand::LocalOperand { name, .. } => locals.get(name),
                        _ => None,
                    };
                    if let Some(op) = match &instr {
                        llvm_ir::Instruction::GetElementPtr(i) => get_local(&i.address),
                        llvm_ir::Instruction::PtrToInt(i) => get_local(&i.operand),
                        llvm_ir::Instruction::IntToPtr(i) => get_local(&i.operand),
                        llvm_ir::Instruction::BitCast(i) => get_local(&i.operand),
                        _ => None,
                    } {
                        locals.insert(n, op.clone());
                    } else {
                        locals.insert(
                            n,
                            Arc::new(Operand::Local(Arc::new(LocalName::Instruction(inst_name)))),
                        );
                    }
                }
            }

            {
                // NOTE! It is important that this is one of three sites where
                // `UArc::new(InstructionName)` is called (and these callsites
                // don't create the same `InstructionName`s), see docs for
                // `UArc`.
                let term_name = UArc::new(InstructionName::new(
                    name.clone(),
                    block_name.clone(),
                    b.instrs.len() + 1,
                ));
                if let Some(n) = b.term.try_get_result() {
                    instructions.insert(n, term_name.clone());
                    locals.insert(
                        n,
                        Arc::new(Operand::Local(Arc::new(LocalName::Instruction(term_name)))),
                    );
                }
            }

            block_names.push(block_name);
        }

        // This has to happen in a subsequent loop because LLVM sometimes
        // references variables before their definitions...
        let mut blocks = Vec::with_capacity(f.basic_blocks.len());
        for (i, b) in f.basic_blocks.iter().enumerate() {
            blocks.push(Block::new(
                operands,
                globals,
                &locals,
                &instructions,
                m,
                b,
                name,
                block_names[i].clone(),
            )?);
        }

        Ok(Function {
            parameters,
            blocks,
            return_type: f.return_type.clone(),
        })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Decl {
    pub parameters: Vec<llvm_ir::TypeRef>,
    pub return_type: llvm_ir::TypeRef,
}

impl Decl {
    pub(crate) fn new<'module>(
        _m: &'module llvm_ir::Module,
        d: &'module llvm_ir::function::FunctionDeclaration,
    ) -> Result<Self, Error> {
        Ok(Decl {
            parameters: d.parameters.iter().map(|p| p.ty.clone()).collect(),
            return_type: d.return_type.clone(),
        })
    }

    pub(crate) fn has_pointer(&self) -> bool {
        if matches!(*self.return_type, llvm_ir::Type::PointerType { .. }) {
            return true;
        }
        for param in &self.parameters {
            if matches!(**param, llvm_ir::Type::PointerType { .. }) {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Hash)]
pub struct Global {
    pub(crate) initializer: Option<Arc<Constant>>,
    pub(crate) is_const: bool,
    pub(crate) ty: llvm_ir::TypeRef,
}

impl Global {
    pub(crate) fn new<'module>(
        globals: &HashMap<&str, Arc<Constant>>,
        _m: &'module llvm_ir::Module,
        g: &'module llvm_ir::module::GlobalVariable,
    ) -> Result<Self, Error> {
        Ok(Global {
            initializer: match &g.initializer {
                None => None,
                Some(i) => Some(Constant::create(globals, i)?),
            },
            is_const: g.is_constant,
            ty: g.ty.clone(),
        })
    }

    /// In bytes
    pub(crate) fn size(&self) -> Option<u64> {
        // TODO: Properly support type sizes
        // https://github.com/cdisselkoen/llvm-ir/issues/31
        if let llvm_ir::Type::PointerType { pointee_type, .. } = &*self.ty {
            // Globals always have pointer types at the top level, so look
            // inside...
            if let llvm_ir::Type::PointerType { .. } = &**pointee_type {
                Some(8)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Module {
    pub decls: HashMap<UArc<FunctionName>, Decl>,
    pub functions: HashMap<UArc<FunctionName>, Function>,
    pub globals: HashMap<Arc<GlobalName>, Global>,
}

impl Module {
    /// Collect names of functions, global variables into Arcs so they can
    /// be shared
    fn global_names(m: &llvm_ir::Module) -> HashMap<&str, Arc<Constant>> {
        let mut global_names: HashMap<&str, Arc<Constant>> = HashMap::with_capacity(
            m.functions.len() + m.func_declarations.len() + m.global_vars.len(),
        );
        for f in &m.functions {
            global_names.insert(
                f.name.as_ref(),
                // NOTE! It is *crucial* that this is the only callsite of
                // FunctionName::definition. This ensures that all other
                // instances of `Arc<FunctionName>` are copies of this pointer,
                // i.e., the underlying function name is not duplicated in
                // memory.
                Arc::new(Constant::Function(UArc::new(FunctionName::definition(f)))),
            );
        }
        for f in &m.func_declarations {
            global_names.insert(
                f.name.as_ref(),
                // See above NOTE!
                Arc::new(Constant::Function(UArc::new(FunctionName::declaration(f)))),
            );
        }
        for g in &m.global_vars {
            global_names.insert(
                g.name.as_ref(),
                // Globals are added by signatures during analysis, so use Arc
                // rather than UArc.
                Arc::new(Constant::Global(Arc::new(GlobalName::new(g)))),
            );
        }
        for g in &m.global_aliases {
            global_names.insert(
                g.name.as_ref(),
                // Globals are added by signatures during analysis, so use Arc
                // rather than UArc.
                Arc::new(Constant::Global(Arc::new(GlobalName::alias(g)))),
            );
        }
        global_names
    }

    pub fn new<'module>(
        m: &'module llvm_ir::Module,
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
    ) -> Result<Self, Error> {
        let global_names = Self::global_names(m);
        let mut functions: HashMap<UArc<FunctionName>, Function> =
            HashMap::with_capacity(m.functions.len());
        for f in &m.functions {
            // This is a hack, but... gotta not duplicate those strings!
            let name = match global_names.get::<str>(f.name.as_ref()) {
                Some(constant) => match &**constant {
                    Constant::Function(f) => f.clone(),
                    _ => unreachable!("Whoops"),
                },
                _ => unreachable!("Whoops"),
            };
            let func = Function::new(operands, &global_names, m, f, &name)?;
            functions.insert(name, func);
        }

        let mut globals: HashMap<Arc<GlobalName>, Global> =
            HashMap::with_capacity(m.global_vars.len());
        for g in &m.global_vars {
            globals.insert(
                // This is a hack, but... gotta not duplicate those strings!
                match global_names.get::<str>(g.name.as_ref()) {
                    Some(constant) => match &**constant {
                        Constant::Global(g) => g.clone(),
                        _ => unreachable!("Whoops"),
                    },
                    _ => unreachable!("Whoops"),
                },
                Global::new(&global_names, m, g)?,
            );
        }

        let mut decls: HashMap<UArc<FunctionName>, Decl> =
            HashMap::with_capacity(m.func_declarations.len());
        for d in &m.func_declarations {
            decls.insert(
                // This is a hack, but... gotta not duplicate those strings!
                match global_names.get::<str>(d.name.as_ref()) {
                    Some(constant) => match &**constant {
                        Constant::Function(f) => f.clone(),
                        _ => unreachable!("Whoops"),
                    },
                    _ => unreachable!("Whoops"),
                },
                Decl::new(m, d)?,
            );
        }

        Ok(Module {
            decls,
            functions,
            globals,
        })
    }
}
