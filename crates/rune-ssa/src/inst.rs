use std::fmt;

use crate::{BlockId, ConstId, PhiId, Program, VarId};
use runestick::ConstValue;

/// Instructions that can be associated with ssa values.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SsaValue {
    /// Undefined value.
    Undef,
    /// Assign a constant value to a variable.
    Const(ConstId),
    /// A phi block.
    Phi(PhiId),
    /// A binary operation.
    Op(SsaOp, VarId, VarId),
}

impl SsaValue {
    /// Debug using the given procedure builder.
    pub fn debug<'a>(&'a self, proc: &'a Program) -> SsaValueDebug<'a> {
        SsaValueDebug { value: self, proc }
    }
}

impl fmt::Debug for SsaValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SsaValue::Undef => {
                write!(f, "undef")
            }
            SsaValue::Const(c) => {
                write!(f, "const({:?})", c)
            }
            SsaValue::Phi(phi_id) => {
                write!(f, "Φ(*)")
            }
            SsaValue::Op(op, a, b) => {
                write!(f, "{:?} {:?} {:?}", a, op, b)
            }
        }
    }
}

pub struct SsaValueDebug<'a> {
    value: &'a SsaValue,
    proc: &'a Program,
}

impl fmt::Debug for SsaValueDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            SsaValue::Undef => {
                write!(f, "undef")
            }
            SsaValue::Const(c) => {
                if let Some(c) = self.proc.get_constant(*c) {
                    write!(f, "{:?}", c)
                } else {
                    write!(f, "const(?)")
                }
            }
            SsaValue::Phi(phi_id) => {
                write!(f, "Φ(")?;

                let mut it = self.proc.phis.operands_of(*phi_id).iter();
                let last = it.next_back();

                for (b, v) in it {
                    write!(f, "v{}_{}", b, v)?;
                    write!(f, ", ")?;
                }

                if let Some((b, v)) = last {
                    write!(f, "v{}_{}", b, v)?;
                }

                write!(f, ")")?;
                Ok(())
            }
            SsaValue::Op(op, a, b) => {
                write!(f, "v{:?} {:?} v{:?}", a, op, b,)
            }
        }
    }
}

/// The kind of an ssa operation.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SsaOp {
    /// Addition.
    Add,
    /// Subtraction.
    Sub,
    /// Division.
    Div,
    /// Multiplication.
    Mul,
}

impl fmt::Debug for SsaOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SsaOp::Add => write!(f, "+"),
            SsaOp::Sub => write!(f, "-"),
            SsaOp::Div => write!(f, "/"),
            SsaOp::Mul => write!(f, "*"),
        }
    }
}
