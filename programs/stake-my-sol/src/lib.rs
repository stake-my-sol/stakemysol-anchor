use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::{invoke, invoke_signed},
    stake as Stake, system_instruction, vote as Vote,
};

declare_id!("CyUJ4YK5NoRopFsffbi4yKCNCScU9rCdtJ3dXmamfui1");

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

        msg!("validating passed stake program");
        require_keys_eq!(stake_program.key(), Stake::program::id());

        msg!(
            "validating passed accounts to be even (i.e in pairs of vote pubkey and stake pubkey)"
        );
        if remaining_accounts.len() % 2 != 0 {
            return Err(ErrorCode::InvalidNumberOfAdditionalAccounts.into());
        }

        let number_of_stake_accouns = (remaining_accounts.len()) / 2;
        let stake_amount_per_account = total_stake_amount / number_of_stake_accouns as u64;

        let vote_pubkeys = remaining_accounts
            .iter()
            .take(number_of_stake_accouns)
            .collect::<Vec<&AccountInfo>>();

        let stake_pubkeys = remaining_accounts
            .iter()
            .skip(number_of_stake_accouns)
            .collect::<Vec<&AccountInfo>>();
        let mut seed_index = initial_seed_index;

        for i in 0..number_of_stake_accouns {
            let current_vote_account = vote_pubkeys[i];
            // if 0 <= i < number_of_stake_accouns then
            // account info in index "i" would be a vote pubkey and
            // its respective stake pubkey would be in
            // "(i + number_of_stake_accouns)th" index in remaining accounts
            let repective_input_stake_pubkey = stake_pubkeys[i];

            msg!("validating provided vote pubkeys");
            require_keys_eq!(*remaining_accounts[i].owner, Vote::program::id());

            msg!("Creating the stake pubkeys");
            let current_seed = &format!("{}-{}", prefix_seed, seed_index);

            let calculated_stake_pubkey =
                Pubkey::create_with_seed(&staker.key(), current_seed, &stake_program.key())
                    .unwrap();

            msg!("Check if the respective input stake pubkey is equal to calculated one");
            require_keys_eq!(repective_input_stake_pubkey.key(), calculated_stake_pubkey);

            // Todo: check if the stake account has already been used

            msg!("Creating the stake account");
            invoke_signed(
                &system_instruction::create_account_with_seed(
                    &staker.key(),
                    &repective_input_stake_pubkey.key(),
                    &staker.key(),
                    current_seed,
                    stake_amount_per_account,
                    std::mem::size_of::<Stake::state::StakeState>() as u64,
                    &stake_program.key(),
                ),
                &[
                    staker.to_account_info(),
                    repective_input_stake_pubkey.to_account_info(),
                    stake_program.to_account_info(),
                ],
                &[&[current_seed.as_bytes()]],
            )?;

            // *: An alternative implemetation
            // system_program::create_account_with_seed(
            //     CpiContext::new(
            //         stake_program.to_account_info(),
            //         system_program::CreateAccountWithSeed {
            //             from: staker.to_account_info(),
            //             to: repective_input_stake_pubkey.to_account_info(),
            //             base: staker.to_account_info(),
            //         },
            //     ),
            //     current_seed,
            //     stake_amount_per_account,
            //     std::mem::size_of::<Stake::state::StakeState>() as u64,
            //     &stake_program.key(),
            // )?;

            msg!("Initializing the stake account");
            invoke(
                &Stake::instruction::initialize(
                    &repective_input_stake_pubkey.key(),
                    &Stake::state::Authorized {
                        staker: staker.key(),
                        withdrawer: staker.key(),
                    },
                    &Stake::state::Lockup {
                        unix_timestamp: 0,
                        epoch: 0,
                        custodian: staker.key(),
                    },
                ),
                &[repective_input_stake_pubkey.to_account_info()],
            )?;

            msg!("Delegating to current vote pubkey");
            invoke(
                &Stake::instruction::delegate_stake(
                    &repective_input_stake_pubkey.key(),
                    &staker.key(),
                    &current_vote_account.key(),
                ),
                &[
                    repective_input_stake_pubkey.to_account_info(),
                    staker.to_account_info(),
                    current_vote_account.to_account_info(),
                    stake_program.to_account_info(), // !: not sure about this one!
                ],
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
    /// CHECK: There is no anchor native validation for the stake program
    stake_program: AccountInfo<'info>,
    #[account(mut)]
    staker: Signer<'info>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("number of additional accounts must be even")]
    InvalidNumberOfAdditionalAccounts,
}
