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
    ) -> Result<()> {
        let number_of_stake_accouns = ctx.remaining_accounts.len() as u64;
        let stake_account_amount = total_stake_amount / number_of_stake_accouns as u64;
        let mut seed_index = initial_seed_index;

        for vote_pubkey in ctx.remaining_accounts.iter() {
            // validate vote pubkeys
            require_keys_eq!(*vote_pubkey.owner, Vote::program::id());

            // Todo: Create the stake pubkey
            let stake_pubkey = Pubkey::create_with_seed(
                &ctx.accounts.payer.key(),
                &format!("stake-{}", initial_seed_index),
                ctx.program_id,
            );

            // Todo: check if the stake account already exists

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
    payer: Signer<'info>,
    stake_authority: AccountInfo<'info>,
    withdraw_authority: AccountInfo<'info>,
    // *: Currently there is no anchor validation for Stake Program
    // *: We just check the program ID to make sure it's the Stake Program
    #[account(address = Stake::program::ID)]
    stake_program: AccountInfo<'info>,
}
