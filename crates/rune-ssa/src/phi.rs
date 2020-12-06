use crate::collections::{HashMap, HashSet};
use crate::{BlockId, Error, ErrorKind, SsaValue, VarId};
use std::fmt;

/// The identifier of a constant value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PhiId(usize);

impl fmt::Display for PhiId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A phi block.
#[derive(Debug)]
pub(crate) struct Phi {
    /// The block the operand belongs to.
    pub(crate) block_id: BlockId,
    /// Operands to this phi node.
    pub(crate) operands: Vec<(BlockId, VarId)>,
    /// Users of this phi block (reverse lookup).
    pub(crate) users: Vec<(BlockId, VarId)>,
}

/// Information on blocks.
#[derive(Debug, Default)]
pub(crate) struct Phis {
    /// Phi nodes.
    phis: Vec<Phi>,
    /// List of users for a given node.
    users: HashMap<PhiId, Vec<(BlockId, VarId)>>,
}

impl Phis {
    /// Register a user for the given phi node.
    pub(crate) fn register_use(
        &mut self,
        phi_id: PhiId,
        block_id: BlockId,
        var: VarId,
    ) -> Result<(), Error> {
        let phi = self
            .phis
            .get_mut(phi_id.0)
            .ok_or_else(|| Error::new(ErrorKind::MissingPhi { phi_id }))?;
        phi.users.push((block_id, var));
        Ok(())
    }

    /// Take users of the given phi node.
    pub(crate) fn take_users_of(&mut self, phi_id: PhiId) -> Vec<(BlockId, VarId)> {
        self.users.remove(&phi_id).unwrap_or_default()
    }

    /// Construct a new phi node in the given block.
    pub(crate) fn build(&mut self, block_id: BlockId) -> PhiId {
        let id = PhiId(self.phis.len());

        self.phis.push(Phi {
            block_id,
            operands: Vec::new(),
            users: Default::default(),
        });

        id
    }

    /// Get the phi node belonging to the given id.
    pub(crate) fn get(&self, phi_id: PhiId) -> Result<&Phi, Error> {
        self.phis
            .get(phi_id.0)
            .ok_or_else(|| Error::new(ErrorKind::MissingPhi { phi_id }))
    }

    /// Get the mutable phi node belonging to the given id.
    pub(crate) fn get_mut(&mut self, phi_id: PhiId) -> Result<&mut Phi, Error> {
        self.phis
            .get_mut(phi_id.0)
            .ok_or_else(|| Error::new(ErrorKind::MissingPhi { phi_id }))
    }

    /// Get operands of this phi node.
    pub(crate) fn operands_of(&self, phi_id: PhiId) -> &[(BlockId, VarId)] {
        match self.phis.get(phi_id.0) {
            Some(phi) => &phi.operands[..],
            None => &[],
        }
    }
}
