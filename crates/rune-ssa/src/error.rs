use crate::{BlockId, PhiId, SsaValue, VarId};
use thiserror::Error;

/// Errors that can be raised by this library.
#[derive(Debug, Error)]
#[error("{kind}")]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    /// Construct a new error from the given kind.
    pub fn new<E>(e: E) -> Self
    where
        ErrorKind: From<E>,
    {
        Self::_new(ErrorKind::from(e))
    }

    fn _new(kind: ErrorKind) -> Self {
        Self { kind }
    }

    /// Access the underlying error kind.
    pub fn into_kind(self) -> ErrorKind {
        self.kind
    }
}

/// The kind of errors that can be raised by this library.
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum ErrorKind {
    #[error("global lookup not implemented yet")]
    GlobalLookupNotImplemented,
    #[error("missing block entry")]
    MissingEntry,
    #[error("missing block with id `{block_id:?}`")]
    MissingBlock { block_id: BlockId },
    #[error("block with id `{block_id:?}` already sealed")]
    BlockAlreadySealded { block_id: BlockId },
    #[error("missing phi node with id `{phi_id:?}`")]
    MissingPhi { phi_id: PhiId },
    #[error("registered conflicting assignment `{assign_id:?}`")]
    ConflictingAssignment { assign_id: VarId },
    #[error("failed to look up phi with id `{phi_id:?}` due to bad rewrite {value:?}")]
    BadPhiReroute { phi_id: PhiId, value: SsaValue },
    #[error("failed to seal {block_id:?} due to incomplete phi node {value:?}")]
    IncompletePhiNode { block_id: BlockId, value: SsaValue },
    #[error("missing variable by id {id:?}")]
    MissingVar { id: VarId },
}
