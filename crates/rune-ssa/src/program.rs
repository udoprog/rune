use crate::collections::HashMap;
use crate::{BlockId, Blocks, ConstId, Consts, Error, ErrorKind, PhiId, Phis, SsaValue};
use runestick::ConstValue;
use std::fmt;

/// A variable assigned exactly once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VarId(usize);

impl fmt::Display for VarId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The builder of a procedure.
#[derive(Debug, Default)]
pub struct Program {
    /// Next variable identifier.
    var_id: usize,
    /// Information on blocks being built.
    pub(crate) blocks: Blocks,
    /// Constants known to the procedure.
    consts: Consts,
    /// Storage for phi blocks.
    pub(crate) phis: Phis,
}

impl Program {
    /// Construct a new procedure builder.
    pub fn new() -> Self {
        Program::default()
    }

    /// Get the value for the given constant id.
    pub fn get_constant(&self, const_id: ConstId) -> Option<&ConstValue> {
        self.consts.get(const_id)
    }

    /// Define a new block.
    pub fn block(&mut self) -> BlockId {
        self.blocks.block()
    }

    /// Define a new variable, unassociated with any specific block.
    pub fn var(&mut self) -> VarId {
        let var = VarId(self.var_id);
        self.var_id += 1;
        var
    }

    /// Define a constant value.
    pub fn write_constant(
        &mut self,
        block_id: BlockId,
        var_id: VarId,
        value: ConstValue,
    ) -> Result<(), Error> {
        let const_id = self.consts.constant(value);
        let const_value = SsaValue::Const(const_id);
        self.write(block_id, var_id, const_value)
    }

    /// Add the given predecessor.
    pub fn add_predecessor(&mut self, from: BlockId, to: BlockId) -> Result<(), Error> {
        self.blocks.add_predecessor(from, to)
    }

    /// Allocate a new variable and write it as an assignment.
    pub fn write(
        &mut self,
        block_id: BlockId,
        var_id: VarId,
        value: SsaValue,
    ) -> Result<(), Error> {
        let var = self.var();
        self.write_var(block_id, var_id, value)?;
        Ok(())
    }

    /// Write an already allocated variable as an assignment.
    pub fn write_var(
        &mut self,
        block_id: BlockId,
        var: VarId,
        value: SsaValue,
    ) -> Result<(), Error> {
        if !self.blocks.contains(block_id) {
            return Err(Error::new(ErrorKind::MissingBlock { block_id }));
        }

        if let SsaValue::Phi(phi_id) = value {
            self.phis.register_use(phi_id, block_id, var);
        }

        self.blocks.register_assignment(block_id, var, value)?;
        Ok(())
    }

    /// Seal the given block by id.
    pub fn seal(&mut self, block_id: BlockId) -> Result<(), Error> {
        for (var, value) in self.blocks.take_incomplete_phis(block_id) {
            let phi_id = match value {
                SsaValue::Phi(phi_id) => phi_id,
                value => return Err(Error::new(ErrorKind::IncompletePhiNode { block_id, value })),
            };

            self.add_phi_operands(block_id, var, phi_id)?;
        }

        self.blocks.seal(block_id)?;
        Ok(())
    }

    /// Algorithm 1, push a single instruction into a block.
    pub fn read(&mut self, block_id: BlockId, var: VarId) -> Result<SsaValue, Error> {
        if !self.blocks.contains(block_id) {
            return Err(Error::new(ErrorKind::MissingBlock { block_id }));
        }

        if let Some(value) = self.blocks.try_get_assignment(block_id, var) {
            return Ok(value);
        }

        self.read_recursive(block_id, var)
    }

    /// Algorithm 2, read a value recursively from dependent blocks.
    fn read_recursive(&mut self, block_id: BlockId, var: VarId) -> Result<SsaValue, Error> {
        let value = if !self.blocks.is_sealed(block_id) {
            let phi_id = self.phis.build(block_id);
            let value = SsaValue::Phi(phi_id);
            self.blocks.register_incomplete_phi(block_id, var, value)?;
            self.phis.register_use(phi_id, block_id, var);
            value
        } else if let Some(block_id) = self.blocks.only_predecessor(block_id) {
            self.read(block_id, var)?
        } else {
            let phi_id = self.phis.build(block_id);
            self.write_var(block_id, var, SsaValue::Phi(phi_id))?;
            self.add_phi_operands(block_id, var, phi_id)?
        };

        Ok(value)
    }

    /// Determine phi operands.
    fn add_phi_operands(
        &mut self,
        block_id: BlockId,
        var: VarId,
        phi_id: PhiId,
    ) -> Result<SsaValue, Error> {
        let phi_block_id = self.phis.get(phi_id)?.block_id;

        if let Some(preds) = self.blocks.take_predecessors(phi_block_id) {
            for block_id in &preds {
                let value = self.read(*block_id, var)?;
                self.phis.get_mut(phi_id)?.operands.push((*block_id, var));
            }

            self.blocks.insert_predecessors(phi_block_id, preds);
        }

        self.try_remove_trivial_phi(phi_id)
    }

    /// Try to remove a trivial phi node.
    fn try_remove_trivial_phi(&mut self, phi_id: PhiId) -> Result<SsaValue, Error> {
        let mut same = None;

        let operands = self.phis.get(phi_id)?.operands.clone();

        for (block_id, var) in operands {
            let op = self.blocks.get_assignment(block_id, var)?;

            if Some(op) == same || op == SsaValue::Phi(phi_id) {
                // Unique value or self-reference.
                continue;
            }

            if same.is_some() {
                // The phi merges at least two values: not trivial
                return Ok(SsaValue::Phi(phi_id));
            }

            same = Some(op);
        }

        let users = self.phis.take_users_of(phi_id);

        let same = same.unwrap_or(SsaValue::Undef);

        /// Replace existing uses.
        for (block_id, var) in &users {
            if let Some(expr) = self.blocks.get_assignment_mut(*block_id, *var) {
                *expr = same;
            }
        }

        let phi_users = users
            .clone()
            .iter()
            .filter_map(|(block_id, var)| self.filter_phi_user(*block_id, *var, phi_id))
            .collect::<Vec<_>>();

        for phi_id in phi_users {
            self.try_remove_trivial_phi(phi_id)?;
        }

        // TODO: implement trivial phi removal.
        Ok(same)
    }

    fn filter_phi_user(&mut self, block_id: BlockId, var: VarId, this: PhiId) -> Option<PhiId> {
        if let Some(SsaValue::Phi(new_id)) = self.blocks.try_get_assignment(block_id, var) {
            if new_id == this {
                return None;
            }

            Some(new_id)
        } else {
            None
        }
    }
}
