use bytemuck::{Pod, Zeroable};
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_token::{instructions::Transfer, state::TokenAccount};

use crate::{constants::ESCROW_SEED, states::Escrow};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct MakeInstructionData {
    pub deposit_amount: [u8; 8],
    pub receive_amount: [u8; 8],
    pub bump: u8,
}

impl MakeInstructionData {
    pub const LEN: usize = core::mem::size_of::<MakeInstructionData>();
}

pub fn process_make(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [maker, mint_a, mint_b, maker_ata_a, vault, escrow, system_program, token_program, _] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate program accounts first
    if system_program.key() != &pinocchio_system::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if token_program.key() != &pinocchio_token::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Basic validation
    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let instruction_data = bytemuck::try_from_bytes::<MakeInstructionData>(&data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    // Validate escrow PDA
    let escrow_pda = pubkey::create_program_address(
        &[
            ESCROW_SEED.as_bytes(),
            maker.key().as_ref(),
            &[instruction_data.bump as u8],
        ],
        &crate::ID,
    )?;

    if escrow.key() != &escrow_pda {
        return Err(ProgramError::InvalidAccountData);
    }

    // Create escrow account FIRST
    pinocchio_system::instructions::CreateAccount {
        from: maker,
        to: escrow,
        space: Escrow::LEN as u64,
        lamports: Rent::get()?.minimum_balance(Escrow::LEN),
        owner: &crate::ID,
    }
    .invoke()?;

    // Initialize escrow state
    let escrow_state = Escrow::load(escrow)?;
    escrow_state.maker = *maker.key();
    escrow_state.mint_a = *mint_a.key();
    escrow_state.mint_b = *mint_b.key();
    escrow_state.receive_amount = instruction_data.receive_amount;
    escrow_state.bump = instruction_data.bump;

    // NOW validate vault ownership (after escrow exists)
    let vault_account = TokenAccount::from_account_info(vault)?;
    if vault_account.owner() != escrow.key() {
        return Err(ProgramError::InvalidAccountData);
    }

    // Transfer tokens
    Transfer {
        from: maker_ata_a,
        to: vault,
        authority: maker,
        amount: u64::from_le_bytes(instruction_data.deposit_amount),
    }
    .invoke()?;

    Ok(())
}
