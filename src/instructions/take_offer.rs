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
pub struct TakeOffer {
    pub bump: u8,
}

impl DataLen for TakeOffer {
    const LEN: usize = core::mem::size_of::<TakeOffer>();
}

pub fn take_offer(accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    let [taker, escrow, vault, maker, maker_token_account, taker_token_account, maker_token_mint, taker_token_mint, _] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !taker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    };

    let escrow_state =
        unsafe { load_acc_mut_unchecked::<EscrowState>(escrow.borrow_mut_data_unchecked())? };

    // passed maker should be valid
    if *maker.key() != escrow_state.maker {
        return Err(ProgramError::InvalidAccountData);
    }

    EscrowState::validate_pda(escrow_state.bump, escrow.key(), &escrow_state.maker)?;

    // decimals for token
    let taker_mint_data = taker_token_mint.try_borrow_data()?;
    let taker_decimals = taker_mint_data[44];

    let maker_mint_data = maker_token_mint.try_borrow_data()?;
    let maker_decimals = maker_mint_data[44];

    // 1. transfer token b to maker
    TransferChecked {
        from: taker_token_account,
        mint: taker_token_mint,
        to: maker_token_account,
        authority: taker,
        amount: escrow_state.taker_amount,
        decimals: taker_decimals,
    }
    .invoke()?;

    // 2. transfer maker's token to taker from vault
    let escrow_bump_bytes = [escrow_state.bump];
    let escrow_signer_seeds = [
        Seed::from(EscrowState::ESCROW_SEED.as_bytes()),
        Seed::from(escrow_state.maker.as_ref()),
        Seed::from(&escrow_bump_bytes[..]),
    ];

    let signers = [Signer::from(&escrow_signer_seeds[..])];

    TransferChecked {
        from: vault,
        mint: maker_token_mint,
        to: taker_token_account,
        authority: escrow, //escrow signs for vault
        amount: escrow_state.maker_amount,
        decimals: maker_decimals,
    }
    .invoke_signed(&signers)?;

    // 3. close escrow and vault accounts
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
