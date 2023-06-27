//! Program instructions
use spl_token::instruction::mint_to;
use solana_program::example_mocks::solana_sdk::transaction::Transaction;
use solana_program::example_mocks::solana_sdk::signature::Keypair;
use solana_program::*;
use solana_program::example_mocks::solana_sdk::signature::Signer;
use spl_token::state::Account;
   
use {
    num_enum::{IntoPrimitive, TryFromPrimitive},
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    },
};

/// Instructions supported by the Token Wrap program
#[derive(Clone, Debug, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum TokenWrapInstruction {
    /// Create a wrapped token mint
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[writeable,signer]` Funding account for mint and backpointer (must be a system account)
    /// 1. `[writeable]` Unallocated wrapped mint account to create, address must be:
    ///     `get_wrapped_mint_address(unwrapped_mint_address, wrapped_token_program_id)`
    /// 2. `[writeable]` Unallocated wrapped backpointer account to create
    ///     `get_wrapped_mint_backpointer_address(wrapped_mint_address)`
    /// 3. `[]` Existing unwrapped mint
    /// 4. `[]` System program
    /// 5. `[]` SPL Token program for wrapped mint
    ///
    /// Data expected by this instruction:
    ///   * bool: true = idempotent creation, false = non-idempotent creation
    ///
    CreateMint,

    /// Wrap tokens
    ///
    /// Move a user's unwrapped tokens into an escrow account and mint the same
    /// number of wrapped tokens into the provided account.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[writeable]` Unwrapped token account to wrap
    /// 1. `[writeable]` Escrow of unwrapped tokens, must be owned by:
    ///     `get_wrapped_mint_authority(wrapped_mint_address)`
    /// 2. `[]` Unwrapped token mint
    /// 3. `[writeable]` Wrapped mint, must be initialized, address must be:
    ///     `get_wrapped_mint_address(unwrapped_mint_address, wrapped_token_program_id)`
    /// 4. `[writeable]` Recipient wrapped token account
    /// 5. `[]` Escrow mint authority, address must be:
    ///     `get_wrapped_mint_authority(wrapped_mint)`
    /// 6. `[]` SPL Token program for unwrapped mint
    /// 7. `[]` SPL Token program for wrapped mint
    /// 8. `[signer]` Transfer authority on unwrapped token account
    /// 8..8+M. `[signer]` (Optional) M multisig signers on unwrapped token account
    ///
    /// Data expected by this instruction:
    ///   * little-endian u64 representing the amount to wrap
    ///
    Wrap,

    /// Unwrap tokens
    ///
    /// Burn user wrapped tokens and transfer the same amount of unwrapped tokens
    /// from the escrow account to the provided account.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[writeable]` Wrapped token account to unwrap
    /// 1. `[writeable]` Wrapped mint, address must be:
    ///     `get_wrapped_mint_address(unwrapped_mint_address, wrapped_token_program_id)`
    /// 2. `[writeable]` Escrow of unwrapped tokens, must be owned by:
    ///     `get_wrapped_mint_authority(wrapped_mint_address)`
    /// 3. `[writeable]` Recipient unwrapped tokens
    /// 4. `[]` Unwrapped token mint
    /// 5. `[]` Escrow unwrapped token authority
    ///     `get_wrapped_mint_authority(wrapped_mint)`
    /// 6. `[]` SPL Token program for wrapped mint
    /// 7. `[]` SPL Token program for unwrapped mint
    /// 8. `[signer]` Transfer authority on wrapped token account
    /// 8..8+M. `[signer]` (Optional) M multisig signers on wrapped token account
    ///
    /// Data expected by this instruction:
    ///   * little-endian u64 representing the amount to unwrap
    ///
    Unwrap,
}

/// Create a `CreateMint` instruction. See `TokenWrapInstruction::CreateMint`

pub fn create_mint(
    program_id: &Pubkey,
    funding_account: &Pubkey,
    wrapped_mint: &Pubkey,
    wrapped_backpointer: &Pubkey,
    unwrapped_mint: &Pubkey,
    idempotent: bool,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(*funding_account, true),
        AccountMeta::new(*wrapped_mint, false),
        AccountMeta::new(*wrapped_backpointer, false),
        AccountMeta::new_readonly(*unwrapped_mint, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];
    if idempotent {
        accounts.push(AccountMeta::new_readonly(spl_token_2022::id(), false));
    }
    Instruction {
        program_id: *program_id,
        accounts,
        data: vec![idempotent as u8],
    }
}
// Derive the wrapped mint address from the unwrapped mint address and the
pub fn wrap(
    program_id: &Pubkey,
    unwrapped_token: &Pubkey,
    unwrapped_mint: &Pubkey,
    wrapped_mint: &Pubkey,
    wrapped_token: &Pubkey,
    amount: u64,
    multisig_signers: Option<Vec<Pubkey>>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(*unwrapped_token, false),
        AccountMeta::new(*wrapped_token, false),
        AccountMeta::new_readonly(*unwrapped_mint, false),
        AccountMeta::new_readonly(*wrapped_mint, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_token_2022::id(), false),
    ];
    if let Some(signers) = multisig_signers {
        for signer in signers {
            accounts.push(AccountMeta::new_readonly(signer, true));
        }
    }
    Instruction {
        program_id: *program_id,
        accounts,
        data: amount.to_le_bytes().to_vec(),
    }
}

pub fn unwrap(
    program_id: &Pubkey,
    wrapped_token: &Pubkey,
    wrapped_mint: &Pubkey,
    unwrapped_token: &Pubkey,
    unwrapped_mint: &Pubkey,
    amount: u64,
    multisig_signers: Option<Vec<Pubkey>>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(*wrapped_token, false),
        AccountMeta::new(*unwrapped_token, false),
        AccountMeta::new_readonly(*wrapped_mint, false),
        AccountMeta::new_readonly(*unwrapped_mint, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_token_2022::id(), false),
    ];
    if let Some(signers) = multisig_signers {
        for signer in signers {
            accounts.push(AccountMeta::new_readonly(signer, true));
        }
    }
    Instruction {
        program_id: *program_id,
        accounts,
        data: amount.to_le_bytes().to_vec(),
    }
}

#[cfg(test)]

pub mod tests {
    use super::*;
    use crate::{
        instruction::{TokenWrapInstruction},
    };
    use solana_program::{
        instruction::{AccountMeta, Instruction},
        program_error::ProgramError,
        program_pack::Pack,
        pubkey::Pubkey,
        system_instruction,
    };
    use spl_token::state::{Account, Mint};

    #[test]
    fn test_create_mint() {
        let program_id = Pubkey::new_unique();
        let funding_account = Pubkey::new_unique();
        let wrapped_mint = Pubkey::new_unique();
        let wrapped_backpointer = Pubkey::new_unique();
        let unwrapped_mint = Pubkey::new_unique();

        let mut accounts = vec![
            AccountMeta::new(funding_account, true),
            AccountMeta::new(wrapped_mint, false),
            AccountMeta::new(wrapped_backpointer, false),
            AccountMeta::new_readonly(unwrapped_mint, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ];
        let instruction = Instruction {
            program_id,
            accounts,
            data: vec![0],
        };
        

        let instruction = create_mint(
            &program_id,
            &funding_account,
            &wrapped_mint,
            &wrapped_backpointer,
            &unwrapped_mint,
            false,
        );
    }

    #[test]
    fn test_wrap() {
        let program_id = Pubkey::new_unique();
        let unwrapped_token = Pubkey::new_unique();
        let unwrapped_mint = Pubkey::new_unique();
        let wrapped_mint = Pubkey::new_unique();
        let wrapped_token = Pubkey::new_unique();
        let amount: u64 = 100;
        let multisig_signers = vec![Pubkey::new_unique(), Pubkey::new_unique()];

        let mut accounts = vec![
            AccountMeta::new(unwrapped_token, false),
            AccountMeta::new(wrapped_token, false),
            AccountMeta::new_readonly(unwrapped_mint, false),
            AccountMeta::new_readonly(wrapped_mint, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_token_2022::id(), false),
        ];
        for signer in multisig_signers.clone() {
            accounts.push(AccountMeta::new_readonly(signer, true));
        }
        

        let instruction = wrap(
            &program_id,
            &unwrapped_token,
            &unwrapped_mint,
            &wrapped_mint,
            &wrapped_token,
            amount,
            Some(multisig_signers),
        );
    }
    #[test]
    fn test_unwrap() {
        let program_id = Pubkey::new_unique();
        let wrapped_token = Pubkey::new_unique();
        let wrapped_mint = Pubkey::new_unique();
        let unwrapped_token = Pubkey::new_unique();
        let unwrapped_mint = Pubkey::new_unique();
        let amount: u64 = 100;
        let multisig_signers = vec![Pubkey::new_unique(), Pubkey::new_unique()];

        let mut accounts = vec![
            AccountMeta::new(wrapped_token, false),
            AccountMeta::new(unwrapped_token, false),
            AccountMeta::new_readonly(wrapped_mint, false),
            AccountMeta::new_readonly(unwrapped_mint, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_token_2022::id(), false),
        ];
        for signer in multisig_signers.clone() {
            accounts.push(AccountMeta::new_readonly(signer, true));
        }
        

        let instruction = unwrap(
            &program_id,
            &wrapped_token,
            &wrapped_mint,
            &unwrapped_token,
            &unwrapped_mint,
            amount,
            Some(multisig_signers),
        );
    }
}