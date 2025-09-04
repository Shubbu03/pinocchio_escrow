# Pinocchio Escrow

A decentralized token swap escrow program built with Pinocchio framework for Solana.

## Overview

This escrow program enables trustless token swaps between two parties. Alice can create an offer to trade Token A for Token B, and Bob can take the offer to complete the swap atomically.

## Features

- **Trustless Token Swaps**: No intermediary needed
- **Atomic Execution**: Either both transfers happen or neither does
- **Refund Mechanism**: Makers can reclaim their tokens
- **PDA-based Security**: Uses Program Derived Addresses for secure token custody

## Program Architecture

### Instructions

1. **`make_offer`** - Create an escrow offer
   - Alice deposits Token A into a vault
   - Specifies how much Token B she wants in return
   - Creates escrow state and vault token account

2. **`take_offer`** - Complete the token swap
   - Bob provides Token B to Alice
   - Bob receives Token A from the vault
   - Escrow and vault accounts are closed

3. **`refund`** - Cancel the offer and reclaim tokens
   - Only the maker (Alice) can refund
   - Transfers tokens back from vault to maker
   - Closes escrow and vault accounts

### State Management

```rust
pub struct EscrowState {
    pub maker: Pubkey,                    // Alice (offer creator)
    pub maker_token_mint: Pubkey,         // Token A mint
    pub taker_token_mint: Pubkey,         // Token B mint
    pub maker_amount: u64,                // Amount of Token A deposited
    pub taker_amount: u64,                // Amount of Token B requested
    pub vault: Pubkey,                    // Vault token account address
    pub maker_token_account: Pubkey,      // Alice's token account
    pub bump: u8,                         // Escrow PDA bump
}
```

### PDA Seeds

- **Escrow PDA**: `["escrow", maker_pubkey]`
- **Vault PDA**: `["vault", escrow_pda]`

## Usage Example

### 1. Make Offer
Alice wants to trade 100 USDC for 0.5 SOL:

```rust
// Accounts needed:
// - maker (Alice, signer)
// - escrow (PDA, will be created)
// - vault (PDA, will be created)
// - maker_token_account (Alice's USDC account)
// - maker_token_mint (USDC mint)
// - taker_token_mint (SOL mint)
// - token_program
// - system_program
// - rent_sysvar

MakeOffer {
    deposit_amount: 100_000_000,  // 100 USDC (6 decimals)
    receive_amount: 500_000_000,  // 0.5 SOL (9 decimals)
    escrow_bump: 254,
}
```

### 2. Take Offer
Bob accepts the offer:

```rust
// Accounts needed:
// - taker (Bob, signer)
// - escrow (existing PDA)
// - vault (existing PDA)
// - maker (Alice, receives tokens)
// - maker_token_account (Alice's USDC account)
// - taker_token_account (Bob's SOL account)
// - maker_token_mint (USDC mint)
// - taker_token_mint (SOL mint)
// - token_program

TakeOffer {
    bump: 254,
}
```

### 3. Refund
Alice cancels her offer:

```rust
// Accounts needed:
// - maker (Alice, signer)
// - escrow (existing PDA)
// - vault (existing PDA)
// - maker_token_account (Alice's USDC account)
// - maker_token_mint (USDC mint)
// - token_program

Refund {
    bump: 254,
}
```

## Security Features

- **PDA Validation**: All PDAs are validated against expected seeds
- **Ownership Checks**: Only authorized parties can execute instructions
- **Atomic Swaps**: Both token transfers happen in the same transaction
- **Rent Recovery**: Account rent is returned to the maker

## Built With

- [Pinocchio](https://github.com/firedancer-io/pinocchio) - Lightweight Solana program framework
- [SPL Token](https://spl.solana.com/) - Solana Program Library for token operations
- [Chio](https://github.com/4rjunc/solana-chio) - Pinocchio project scaffolding by [@4rjunc](https://github.com/4rjunc)

## Development

```bash
# Build the program
cargo build-sbf

# Run tests
cargo test

# Deploy to devnet
solana program deploy target/deploy/pinocchio_escrow.so
```

## Program ID

```
GbqefpNQgSDkGj3Yv3zdtUiVD9qgZo6LGw3ZTeBJgbWP
```