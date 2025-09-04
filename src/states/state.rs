use super::utils::DataLen;
use pinocchio::{
    program_error::ProgramError,
    pubkey::{self, Pubkey},
};

use crate::errors::MyProgramError;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EscrowState {
    pub maker: Pubkey,
    pub maker_token_mint: Pubkey,
    pub taker_token_mint: Pubkey,
    pub maker_amount: u64,
    pub taker_amount: u64,
    pub vault: Pubkey,
    pub maker_token_account: Pubkey,
    pub bump: u8,
}

impl DataLen for EscrowState {
    const LEN: usize = core::mem::size_of::<EscrowState>();
}

impl EscrowState {
    pub const ESCROW_SEED: &'static str = "escrow";
    pub const VAULT_SEED: &'static str = "vault";

    pub fn validate_pda(bump: u8, pda: &Pubkey, owner: &Pubkey) -> Result<(), ProgramError> {
        let seed_with_bump = &[Self::ESCROW_SEED.as_bytes(), owner, &[bump]];
        let derived = pubkey::create_program_address(seed_with_bump, &crate::ID)?;
        if derived != *pda {
            return Err(MyProgramError::PdaMismatch.into());
        }
        Ok(())
    }
}
