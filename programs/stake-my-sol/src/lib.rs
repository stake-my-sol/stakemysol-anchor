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
        let remaining_accounts = ctx.remaining_accounts;

        msg!(
            "validating passed accounts to be even (i.e in pairs of vote pubkey and stake pubkey)"
        );
        if remaining_accounts.len() % 2 != 0 {
            return Err(ErrorCode::InvalidNumberOfAdditionalAccounts.into());
        }

        let number_of_stake_accouns = (remaining_accounts.len() as u64) / 2;
        let stake_amount_per_account = total_stake_amount / number_of_stake_accouns as u64;
        let mut seed_index = initial_seed_index;

        for i in 0..number_of_stake_accouns as usize {
            msg!("validating provided vote pubkeys");
            require_keys_eq!(*remaining_accounts[i].owner, Vote::program::id());

            msg!("Creating the stake pubkeys");
            let stake_pubkey = Pubkey::create_with_seed(
                &ctx.accounts.staker.key(),
                &format!("{}-{}", prefix_seed, seed_index),
                ctx.program_id,
            )
            .unwrap();

            msg!("Check if the respective input stake pubkey is equal to calculated one");
            require_keys_eq!(
                *remaining_accounts[i + (number_of_stake_accouns as usize)].owner,
                stake_pubkey
            );

            // Todo: check if the stake account has already been used

            // Todo: Create the stake account and Delegate
            // Stake::instruction::create_account_with_seed_and_delegate_stake(
            //     // from_pubkey,
            //     // stake_pubkey,
            //     // base,
            //     // seed,
            //     // vote_pubkey,
            //     // authorized,
            //     // lockup,
            //     // lamports
            //     CpiContext::new(ctx.accounts.staker.to_account_info(), stake_pubkey),
            // );
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
    // *: Currently there is no anchor validation for Stake Program
    // *: We just check the program ID to make sure it's the Stake Program
    #[account(address = Stake::program::ID)]
    stake_program: AccountInfo<'info>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("number of additional accounts must be even")]
    InvalidNumberOfAdditionalAccounts,
}
