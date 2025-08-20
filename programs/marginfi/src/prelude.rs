use anchor_lang::prelude::*;

// Shorthand for Result<G, ProgramError>
// G = (): If there is no specific return value, it means Result<(), ProgramError>
pub type MarginfiResult<G = ()> = Result<G>;

pub use crate::{errors::MarginfiError, state::marginfi_group::MarginfiGroup};
