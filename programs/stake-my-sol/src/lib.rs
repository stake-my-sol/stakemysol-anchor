use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::{invoke, invoke_signed},
    stake as Stake, system_instruction, vote as Vote,
};
use anchor_lang::system_program;

declare_id!("C6Vb7CQCa2Hovhpdu1iaZR3VBcbKt3W9XNHRYYs1NVd");

#[program]
pub mod stake_my_sol {
    use super::*;

    pub fn create_stake_accounts_and_delegate<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateStakeAccountsAndDelegate<'info>>,
        total_stake_amount: u64,
        initial_seed_index: u8,
        prefix_seed: String,
    ) -> Result<()> {
        let remaining_accounts = ctx.remaining_accounts;
        let staker = &mut ctx.accounts.staker;
        let stake_program = &ctx.accounts.stake_program;
        let system_program = &ctx.accounts.system_program;
        let rent_sysvar = &ctx.accounts.rent_sysvar;
        let clock_sysvar = &ctx.accounts.clock_sysvar;
        let stake_history_sysvar = &ctx.accounts.stake_history_sysvar;
        let stake_config_sysvar = &ctx.accounts.stake_config_sysvar;

        msg!("validating passed stake program");
        require_keys_eq!(stake_program.key(), Stake::program::id());

        msg!("validating passed stake config account");
        require_keys_eq!(stake_config_sysvar.key(), Stake::config::id());

        msg!(
            "validating passed accounts to be even (i.e in pairs of vote pubkey and stake pubkey)"
        );
        if (remaining_accounts.len() % 2 != 0) | (remaining_accounts.len() == 0) {
            return Err(ErrorCode::InvalidNumberOfAdditionalAccounts.into());
        }

        let number_of_stake_accouns = (remaining_accounts.len()) / 2;
        let stake_amount_per_account = total_stake_amount
            .checked_div(number_of_stake_accouns as u64)
            .unwrap();

        let vote_accounts = remaining_accounts
            .iter()
            .take(number_of_stake_accouns)
            .collect::<Vec<&AccountInfo>>();

        msg!("check owner of vote accounts to be Vote::program::id()");
        for vote_acc in vote_accounts.iter() {
            msg!("validating passed vote account");
            require_keys_eq!(*vote_acc.owner, Vote::program::id());
        }

        let stake_accounts = remaining_accounts
            .iter()
            .skip(number_of_stake_accouns)
            .collect::<Vec<&AccountInfo>>();

        for stake_acc in stake_accounts.iter() {
            msg!("validating passed stake accounts");
            // *: stake accounts should not be initialized
            // *: i.e owner should be system_program::id() not the stake_program::id()
            require_keys_neq!(*stake_acc.owner, Stake::program::id());
        }

        let mut seed_index = initial_seed_index;

        for i in 0..number_of_stake_accouns {
            let current_vote_account = vote_accounts[i];
            // if 0 <= i < number_of_stake_accouns then
            // account info in index "i" would be a vote pubkey and
            // its respective stake pubkey would be in
            // "(i + number_of_stake_accouns)th" index in remaining accounts

            let repective_input_stake_account = stake_accounts[i];

            msg!("validating provided vote pubkeys");
            require_keys_eq!(*remaining_accounts[i].owner, Vote::program::id());

            msg!("Creating the stake pubkeys");
            let current_seed = &format!("{}-{}", prefix_seed, seed_index);

            let calculated_stake_pubkey =
                Pubkey::create_with_seed(&staker.key(), current_seed, &stake_program.key())
                    .unwrap();

            msg!("Check if the respective input stake pubkey is equal to calculated one");
            require_keys_eq!(repective_input_stake_account.key(), calculated_stake_pubkey);

            // Todo: check if the stake account has already been used

            msg!("Creating the stake account");

            // invoke_signed(
            //     &system_instruction::create_account_with_seed(
            //         &staker.key(),
            //         &repective_input_stake_account.key(),
            //         &staker.key(),
            //         current_seed,
            //         stake_amount_per_account,
            //         std::mem::size_of::<Stake::state::StakeState>() as u64,
            //         &stake_program.key(),
            //     ),
            //     &[
            //         system_program.to_account_info(),
            //         repective_input_stake_account.to_account_info(),
            //         staker.to_account_info(),
            //     ],
            //     &[&[
            //         staker.key().as_ref(),
            //         current_seed.as_bytes(),
            //         Stake::program::id().as_ref(),
            //     ]],
            // )?;

            // *: An alternative implemetation
            system_program::create_account_with_seed(
                CpiContext::new_with_signer(
                    stake_program.to_account_info(),
                    system_program::CreateAccountWithSeed {
                        from: staker.to_account_info(),
                        to: repective_input_stake_account.to_account_info(),
                        base: staker.to_account_info(),
                    },
                    &[&[
                        staker.key().as_ref(),
                        current_seed.as_bytes(),
                        Stake::program::id().as_ref(),
                    ]],
                ),
                current_seed,
                stake_amount_per_account,
                std::mem::size_of::<Stake::state::StakeState>() as u64,
                &stake_program.key(),
            )?;

            msg!("Initializing the stake account");
            invoke(
                &Stake::instruction::initialize(
                    &repective_input_stake_account.key(),
                    &Stake::state::Authorized {
                        staker: staker.key(),
                        withdrawer: staker.key(),
                    },
                    &Stake::state::Lockup::default(),
                ),
                &[
                    stake_program.to_account_info(),
                    repective_input_stake_account.to_account_info(),
                    rent_sysvar.to_account_info(),
                ],
            )?;

            msg!("Delegating to current vote pubkey");
            invoke_signed(
                &Stake::instruction::delegate_stake(
                    &repective_input_stake_account.key(),
                    &staker.key(),
                    &current_vote_account.key(),
                ),
                &[
                    stake_program.to_account_info(),
                    repective_input_stake_account.to_account_info(),
                    staker.to_account_info(),
                    current_vote_account.to_account_info(),
                    clock_sysvar.to_account_info(),
                    stake_history_sysvar.to_account_info(),
                    stake_config_sysvar.to_account_info(),
                ],
                &[&[
                    staker.key().as_ref(),
                    current_seed.as_bytes(),
                    Stake::program::id().as_ref(),
                ]],
            )?;

            seed_index += 1;
        }

        Ok(())
    }
}

// *: Currently there is no anchor validation for Stake Program
// *: We just check the program ID to make sure it's the Stake Program
#[derive(Accounts)]
pub struct CreateStakeAccountsAndDelegate<'info> {
    /// CHECK: validated inside instruction
    stake_program: AccountInfo<'info>,
    #[account(mut)]
    staker: Signer<'info>,
    system_program: Program<'info, System>,
    rent_sysvar: Sysvar<'info, Rent>,
    clock_sysvar: Sysvar<'info, Clock>,
    stake_history_sysvar: Sysvar<'info, StakeHistory>,
    /// CHECK: validated inside instruction
    stake_config_sysvar: AccountInfo<'info>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("number of additional accounts must be even")]
    InvalidNumberOfAdditionalAccounts,
}
