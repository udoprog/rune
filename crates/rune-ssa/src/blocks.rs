use crate::collections::HashMap;
use crate::{Error, ErrorKind, SsaValue, VarId};
use std::fmt;

/// Any informatin needed about blocks.
#[derive(Debug)]
pub struct Block {
    /// The id of the block.
    pub id: BlockId,
    /// If the block is sealed or not.
    pub sealed: bool,
    /// All assignments in this block.
    pub assignments: HashMap<VarId, SsaValue>,
    /// Predecessor blocks.
    predecessors: Vec<BlockId>,
    /// Incomplete phis in this block.
    incomplete_phis: Vec<(VarId, SsaValue)>,
}

/// Information on blocks.
#[derive(Debug, Default)]
pub(crate) struct Blocks {
    blocks: Vec<Block>,
}

impl Blocks {
    /// Add a predecessor block to this block.
    pub fn add_predecessor(&mut self, from: BlockId, to: BlockId) -> Result<(), Error> {
        let block = self
            .blocks
            .get_mut(from.0)
            .ok_or_else(|| Error::new(ErrorKind::MissingBlock { block_id: from }))?;
        block.predecessors.push(to);
        Ok(())
    }

    /// Take incomplete phis for the given block.
    pub(crate) fn take_incomplete_phis(&mut self, block_id: BlockId) -> Vec<(VarId, SsaValue)> {
        match self.blocks.get_mut(block_id.0) {
            Some(block) => std::mem::take(&mut block.incomplete_phis),
            None => Vec::new(),
        }
    }

    /// Take incomplete phis for the given block.
    pub(crate) fn register_incomplete_phi(
        &mut self,
        block_id: BlockId,
        var: VarId,
        value: SsaValue,
    ) -> Result<(), Error> {
        let block = self
            .blocks
            .get_mut(block_id.0)
            .ok_or_else(|| Error::new(ErrorKind::MissingBlock { block_id }))?;

        block.incomplete_phis.push((var, value));
        Ok(())
    }

    /// Register an assignment.
    pub(crate) fn register_assignment(
        &mut self,
        block_id: BlockId,
        var: VarId,
        value: SsaValue,
    ) -> Result<(), Error> {
        let block = self
            .blocks
            .get_mut(block_id.0)
            .ok_or_else(|| Error::new(ErrorKind::MissingBlock { block_id }))?;

        block.assignments.insert(var, value);
        Ok(())
    }

    /// Get the given assignment.
    pub(crate) fn get_assignment(&self, block_id: BlockId, var: VarId) -> Result<SsaValue, Error> {
        let block = self
            .blocks
            .get(block_id.0)
            .ok_or_else(|| Error::new(ErrorKind::MissingBlock { block_id }))?;
        Ok(*block
            .assignments
            .get(&var)
            .ok_or_else(|| Error::new(ErrorKind::MissingVar { id: var }))?)
    }

    /// Get the given assignment.
    pub(crate) fn try_get_assignment(&self, block_id: BlockId, var: VarId) -> Option<SsaValue> {
        let block = self.blocks.get(block_id.0)?;
        Some(*block.assignments.get(&var)?)
    }

    /// Get the given assignment.
    pub(crate) fn get_assignment_mut(
        &mut self,
        block_id: BlockId,
        var: VarId,
    ) -> Option<&mut SsaValue> {
        let block = self.blocks.get_mut(block_id.0)?;
        block.assignments.get_mut(&var)
    }

    /// Test if we contain the given block.
    pub(crate) fn contains(&self, block_id: BlockId) -> bool {
        self.blocks.get(block_id.0).is_some()
    }

    /// Test if the block is sealed.
    pub(crate) fn is_sealed(&self, block_id: BlockId) -> bool {
        match self.blocks.get(block_id.0) {
            Some(block) => block.sealed,
            None => false,
        }
    }

    /// Seal the given block.
    pub(crate) fn seal(&mut self, block_id: BlockId) -> Result<(), Error> {
        let block = self
            .blocks
            .get_mut(block_id.0)
            .ok_or_else(|| Error::new(ErrorKind::MissingBlock { block_id }))?;

        if block.sealed {
            return Err(Error::new(ErrorKind::BlockAlreadySealded { block_id }));
        }

        block.sealed = true;
        Ok(())
    }

    /// Create a new empty block and return its identifier.
    pub(crate) fn block(&mut self) -> BlockId {
        let id = BlockId(self.blocks.len());

        self.blocks.push(Block {
            id,
            sealed: false,
            assignments: Default::default(),
            predecessors: Default::default(),
            incomplete_phis: Default::default(),
        });

        id
    }

    /// Gets the only predecessor for the given block, if it only has one.
    pub(crate) fn only_predecessor(&self, id: BlockId) -> Option<BlockId> {
        let block = self.blocks.get(id.0)?;

        if block.predecessors.len() == 1 {
            block.predecessors.first().copied()
        } else {
            None
        }
    }

    /// Take predecessors for the given block.
    pub(crate) fn take_predecessors(&mut self, id: BlockId) -> Option<Vec<BlockId>> {
        match self.blocks.get_mut(id.0) {
            Some(block) => Some(std::mem::take(&mut block.predecessors)),
            None => None,
        }
    }

    /// Insert predecessors for the given block, if it exists.
    pub(crate) fn insert_predecessors(&mut self, id: BlockId, predecessors: Vec<BlockId>) {
        if let Some(block) = self.blocks.get_mut(id.0) {
            block.predecessors = predecessors;
        }
    }
}

impl<'a> IntoIterator for &'a Blocks {
    type Item = &'a Block;
    type IntoIter = std::slice::Iter<'a, Block>;

    fn into_iter(self) -> Self::IntoIter {
        self.blocks.iter()
    }
}

/// The identifier of a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct BlockId(usize);

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
