use runestick::ConstValue;
use std::fmt;

/// The identifier of a constant value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct ConstId(usize);

impl fmt::Display for ConstId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Information on blocks.
#[derive(Debug, Default)]
pub(crate) struct Consts {
    consts: Vec<ConstValue>,
}

impl Consts {
    /// Insert the given constant and return its identifier.
    pub(crate) fn constant(&mut self, value: ConstValue) -> ConstId {
        let const_id = ConstId(self.consts.len());
        self.consts.push(value);
        const_id
    }

    /// Get the constant value.
    pub(crate) fn get(&self, id: ConstId) -> Option<&ConstValue> {
        self.consts.get(id.0)
    }
}
