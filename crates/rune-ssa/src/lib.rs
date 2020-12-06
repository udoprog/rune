//! <div align="center">
//!     <img alt="Rune Logo" src="https://raw.githubusercontent.com/rune-rs/rune/main/assets/icon.png" />
//! </div>
//!
//! <br>
//!
//! <div align="center">
//! <a href="https://rune-rs.github.io">
//!     <b>Visit the site 🌐</b>
//! </a>
//! -
//! <a href="https://rune-rs.github.io/book/">
//!     <b>Read the book 📖</b>
//! </a>
//! </div>
//!
//! <br>
//!
//! <div align="center">
//! <a href="https://github.com/rune-rs/rune/actions">
//!     <img alt="Build Status" src="https://github.com/rune-rs/rune/workflows/Build/badge.svg">
//! </a>
//!
//! <a href="https://github.com/rune-rs/rune/actions">
//!     <img alt="Site Status" src="https://github.com/rune-rs/rune/workflows/Site/badge.svg">
//! </a>
//!
//! <a href="https://crates.io/crates/rune">
//!     <img alt="crates.io" src="https://img.shields.io/crates/v/rune.svg">
//! </a>
//!
//! <a href="https://docs.rs/rune">
//!     <img alt="docs.rs" src="https://docs.rs/rune/badge.svg">
//! </a>
//!
//! <a href="https://discord.gg/v5AeNkT">
//!     <img alt="Chat on Discord" src="https://img.shields.io/discord/558644981137670144.svg?logo=discord&style=flat-square">
//! </a>
//! </div>
//!
//! <br>
//!
//! The SSA component of Rune, and embeddable scripting language for Rust.

#![deny(missing_docs)]
#![allow(unused)]

mod blocks;
mod consts;
mod error;
mod inst;
mod phi;
mod program;

pub use self::blocks::BlockId;
pub(crate) use self::blocks::Blocks;
pub use self::consts::ConstId;
pub(crate) use self::consts::Consts;
pub use self::error::{Error, ErrorKind};
pub use self::inst::{SsaOp, SsaValue};
pub(crate) use self::phi::{PhiId, Phis};
pub use self::program::{Program, VarId};

pub(crate) mod collections {
    pub(crate) use ::hashbrown::HashMap;
    pub(crate) use ::hashbrown::HashSet;
}

#[cfg(test)]
mod tests {
    use crate::{Error, Program, SsaOp, SsaValue};
    use runestick::ConstValue;

    #[test]
    fn test_fig_1() -> Result<(), Error> {
        let mut proc = Program::new();
        let entry = proc.block();

        proc.seal(entry)?;

        for block in &proc.blocks {
            println!("block{:?}:", block.id);

            let mut vars = block.assignments.iter().collect::<Vec<_>>();
            vars.sort();

            for (id, value) in vars {
                println!("  v{:?} <- {:?}", id, value.debug(&proc));
            }
        }

        Ok(())
    }

    #[test]
    fn test_forking() -> Result<(), Error> {
        let mut program = Program::new();

        let a = program.var();
        let b = program.var();
        let c = program.var();

        let block0 = program.block();
        let block1 = program.block();
        let block2 = program.block();

        {
            program.write_constant(block0, a, ConstValue::Integer(1))?;
            program.seal(block0)?;
        }

        {
            program.write_constant(block1, a, ConstValue::Integer(2))?;
            program.seal(block1)?;
        }

        {
            program.add_predecessor(block2, block0);
            program.add_predecessor(block2, block1);
            let v = program.read(block2, a)?;
            program.write_var(block2, a, v)?;

            program.seal(block2)?;
        }

        for block in &program.blocks {
            println!("block{}:", block.id);

            let mut vars = block.assignments.iter().collect::<Vec<_>>();
            vars.sort();

            for (id, value) in vars {
                println!("  v{}_{} <- {:?}", block.id, id, value.debug(&program));
            }
        }

        Ok(())
    }
}
