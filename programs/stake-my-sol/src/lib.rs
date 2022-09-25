use anchor_lang::prelude::*;
use anchor_lang::solana_program::stake as Stake;
use anchor_lang::solana_program::vote as Vote;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod stake_my_sol {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn create_stake_accounts_and_delegate(
        ctx: Context<CreateStakeAccountsAndDelegate>,
        total_stake_amount: u64,
        initial_seed_index: u32,
        prefix_seed: String,
    ) -> Result<()> {
        let number_of_stake_accouns = ctx.remaining_accounts.len() as u64;
        let stake_account_amount = total_stake_amount / number_of_stake_accouns as u64;
        let mut seed_index = initial_seed_index;

        for vote_pubkey in ctx.remaining_accounts.iter() {
            // validate vote pubkeys
            require_keys_eq!(*vote_pubkey.owner, Vote::program::id());

            // Create the stake pubkey
            let stake_pubkey = Pubkey::create_with_seed(
                &ctx.accounts.staker.key(),
                &format!("{}-{}", prefix_seed, seed_index),
                ctx.program_id,
            )
            .unwrap();

            // Todo: check if the stake account has already been used

            // Todo: Create the stake account and Delegate

            // Todo: Stake::create_with_seed_and_delegate_stake

            seed_index += 1;
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct CreateStakeAccountsAndDelegate<'info> {
    #[account(mut)]
    staker: Signer<'info>,
    stake_authority: AccountInfo<'info>,
    withdraw_authority: AccountInfo<'info>,
    // *: Currently there is no anchor validation for Stake Program
    // *: We just check the program ID to make sure it's the Stake Program
    #[account(address = Stake::program::ID)]
    stake_program: AccountInfo<'info>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("The provided vote account is not owned by the Vote Program")]
    InvalidVoteAccount,
    #[msg("The provided stake account is not owned by the Stake Program")]
    InvalidStakeAccount,
    #[msg("The provided stake account is already delegated")]
    StakeAccountAlreadyDelegated,
    #[msg("The provided stake account is already initialized")]
    StakeAccountAlreadyInitialized,
    #[msg("Failed to create a public key from seed")]
    FailedToCreatePubkeyFromSeed,
}
