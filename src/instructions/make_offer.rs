use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::find_program_address,
    sysvars::rent::Rent,
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::{InitializeAccount, TransferChecked};

use crate::states::{load_ix_data, utils::load_acc_mut_unchecked, DataLen, EscrowState};

#[repr(C)]
pub struct MakeOffer {
    pub deposit_amount: u64,
    pub receive_amount: u64,
    pub escrow_bump: u8,
}

impl DataLen for MakeOffer {
    const LEN: usize = core::mem::size_of::<MakeOffer>();
}

pub fn make_offer(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [maker, escrow, vault, maker_token_account, maker_token_mint, taker_token_mint, token_program, rent_sysvar, _] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    };

    let make_offer_ix_data = unsafe { load_ix_data::<MakeOffer>(data)? };

    if make_offer_ix_data.deposit_amount == 0 || make_offer_ix_data.receive_amount == 0 {
        return Err(ProgramError::InvalidInstructionData);
    };

    EscrowState::validate_pda(make_offer_ix_data.escrow_bump, escrow.key(), maker.key())?;

    let rent = Rent::from_account_info(rent_sysvar)?;

    let bump_bytes = [make_offer_ix_data.escrow_bump];

    // 1. create escrow account
    let escrow_signer_seeds = [
        Seed::from(EscrowState::ESCROW_SEED.as_bytes()),
        Seed::from(maker.key().as_ref()),
        Seed::from(&bump_bytes[..]),
    ];

    let escrow_signers = [Signer::from(&escrow_signer_seeds[..])];

    CreateAccount {
        from: maker,
        to: escrow,
        space: EscrowState::LEN as u64,
        owner: &crate::ID,
        lamports: rent.minimum_balance(EscrowState::LEN),
    }
    .invoke_signed(&escrow_signers)?;

    // 2. create vault token account
    let (vault_pda, vault_bump) = find_program_address(
        &[EscrowState::VAULT_SEED.as_bytes(), escrow.key().as_ref()],
        &crate::ID,
    );

    if vault_pda != *vault.key() {
        return Err(ProgramError::InvalidAccountData);
    }

    let vault_bump_bytes = [vault_bump];

    let vault_signer_seeds = [
        Seed::from(EscrowState::VAULT_SEED.as_bytes()),
        Seed::from(escrow.key().as_ref()),
        Seed::from(&vault_bump_bytes[..]),
    ];

    let vault_signers = [Signer::from(&vault_signer_seeds[..])];

    CreateAccount {
        from: maker,
        to: vault,
        space: 165, //spl token account size,
        owner: token_program.key(),
        lamports: rent.minimum_balance(165),
    }
    .invoke_signed(&vault_signers)?;

    // Initialize the vault as a token account
    InitializeAccount {
        account: vault,
        mint: maker_token_mint,
        owner: escrow,
        rent_sysvar: rent_sysvar, // Escrow owns the vault
    }
    .invoke()?;

    // 3. transfer from maker to vault(spl token transfer)
    // for finding decimals from mint account
    let maker_mint_data = maker_token_mint.try_borrow_data()?;

    let decimals = maker_mint_data[44]; // Decimals field in mint account

    TransferChecked {
        from: maker_token_account,
        to: vault,
        authority: maker,
        mint: maker_token_mint,
        amount: make_offer_ix_data.deposit_amount,
        decimals,
    }
    .invoke()?;

    // 4. write escrow state
    let escrow_state =
        unsafe { load_acc_mut_unchecked::<EscrowState>(escrow.borrow_mut_data_unchecked()) }?;

    *escrow_state = EscrowState {
        maker: *maker.key(),
        maker_token_mint: *maker_token_mint.key(),
        taker_token_mint: *taker_token_mint.key(),
        maker_amount: make_offer_ix_data.deposit_amount,
        taker_amount: make_offer_ix_data.receive_amount,
        vault: *vault.key(),
        maker_token_account: *maker_token_account.key(),
        bump: make_offer_ix_data.escrow_bump,
    };

    Ok(())
}
