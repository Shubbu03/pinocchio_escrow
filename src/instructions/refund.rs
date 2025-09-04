use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    ProgramResult,
};
use pinocchio_system::instructions::Transfer;
use pinocchio_token::instructions::{CloseAccount, TransferChecked};

use crate::states::{load_acc_mut_unchecked, DataLen, EscrowState};

#[repr(C)]
pub struct Refund {
    pub bump: u8,
}

impl DataLen for Refund {
    const LEN: usize = core::mem::size_of::<Refund>();
}

pub fn refund(accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    let [maker, escrow, vault, maker_token_account, maker_token_mint, _] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    };

    let escrow_state =
        unsafe { load_acc_mut_unchecked::<EscrowState>(escrow.borrow_mut_data_unchecked())? };

    if *maker.key() != escrow_state.maker {
        return Err(ProgramError::InvalidAccountData);
    }

    EscrowState::validate_pda(escrow_state.bump, escrow.key(), &escrow_state.maker)?;

    // 1. transfer tokens back to maker token acc from vault
    let escrow_bump_bytes = [escrow_state.bump];
    let escrow_signer_seeds = [
        Seed::from(EscrowState::ESCROW_SEED.as_bytes()),
        Seed::from(escrow_state.maker.as_ref()),
        Seed::from(&escrow_bump_bytes[..]),
    ];

    let signers = [Signer::from(&escrow_signer_seeds[..])];

    let maker_mint_data = maker_token_mint.try_borrow_data()?;
    let maker_decimals = maker_mint_data[44];

    TransferChecked {
        from: vault,
        mint: maker_token_mint,
        to: maker_token_account,
        authority: escrow,
        amount: escrow_state.maker_amount,
        decimals: maker_decimals,
    }
    .invoke_signed(&signers)?;

    // 2. close escrow and vault acc
    CloseAccount {
        account: vault,
        destination: maker, // rent back to maker
        authority: escrow,  // Escrow signs
    }
    .invoke_signed(&signers)?;

    Transfer {
        from: escrow,
        to: maker,
        lamports: escrow.lamports(),
    }
    .invoke_signed(&signers)?;

    Ok(())
}
