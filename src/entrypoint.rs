#![allow(unexpected_cfgs)]

use crate::instructions::{self, EscrowInstruction};
use pinocchio::{
    account_info::AccountInfo, default_panic_handler, no_allocator, program_entrypoint,
    program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

// This is the entrypoint for the program.
program_entrypoint!(process_instruction);
//Do not allocate memory.
no_allocator!();
// Use the no_std panic handler.
default_panic_handler!();

#[inline(always)]
fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (ix_disc, instruction_data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match EscrowInstruction::try_from(ix_disc)? {
        EscrowInstruction::MakeOffer => instructions::process_make(accounts, &instruction_data),
        EscrowInstruction::TakeOffer => instructions::process_take(accounts),
        EscrowInstruction::Refund => instructions::process_refund(accounts),
    }
}
