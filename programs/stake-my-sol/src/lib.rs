use anchor_lang::prelude::*;
use anchor_lang::solana_program::stake as Stake;
use anchor_lang::solana_program::vote as Vote;

declare_id!("F5m8b7d8o3PMGyStiMdT6N6cttvMHWiXaJNWQ5Jve94k");

#[program]
pub mod stake_my_sol {
    use super::*;

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
        let mut current_vote_account: &AccountInfo;

        for i in 0..number_of_stake_accouns as usize {
            current_vote_account = &remaining_accounts[i];

            msg!("validating provided vote pubkeys");
            require_keys_eq!(*remaining_accounts[i].owner, Vote::program::id());

            msg!("Creating the stake pubkeys");
            let current_seed = &format!("{}-{}", prefix_seed, seed_index);

            let stake_pubkey = Pubkey::create_with_seed(
                &ctx.accounts.staker.key(),
                current_seed,
                &ctx.accounts.stake_program.key(),
            )
            .unwrap();

            msg!("Check if the respective input stake pubkey is equal to calculated one");
            // if 0 <= i < number_of_stake_accouns then
            // account info in index "i" would be a vote pubkey and
            // its respective stake pubkey would be in
            // "(i + number_of_stake_accouns)th" index in remaining accounts
            require_keys_eq!(
                *remaining_accounts[i + (number_of_stake_accouns as usize)].owner,
                stake_pubkey
            );

            // Todo: check if the stake account has already been used
            msg!("Create stake account and delegate");
            Stake::instruction::create_account_with_seed_and_delegate_stake(
                &ctx.accounts.staker.key(),
                &stake_pubkey,
                &ctx.accounts.staker.key(),
                current_seed,
                &current_vote_account.key(),
                &Stake::state::Authorized {
                    staker: ctx.accounts.staker.key(),
                    withdrawer: ctx.accounts.staker.key(),
                },
                &Stake::state::Lockup {
                    unix_timestamp: 0,
                    epoch: 0,
                    custodian: ctx.accounts.staker.key(),
                },
                stake_amount_per_account,
            );

            // increment seed_index for next iteration
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
