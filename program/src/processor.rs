use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction::{self},
    sysvar::{rent::Rent, Sysvar},
};

use spl_token::state::{Account, Mint};

use crate::{
    account::{SimpleTreasury, SimpleTreasuryMode, VestedTreasury},
    error::TreasuryError,
    instruction::TreasuryInstruction,
};

/// Verify an Associated Account
#[macro_export]
macro_rules! verify_associated {
    ($assoc:expr, $owner:expr, $mint:expr) => {
        match Account::unpack(&$assoc.data.borrow()) {
            Ok(account) => {
                if account.mint != $mint {
                    Err(TreasuryError::MintWrongToken.into())
                } else if account.owner != $owner {
                    Err(TreasuryError::InvalidAssociatedAccount.into())
                } else {
                    Ok(account)
                }
            }
            _ => Err(TreasuryError::InvalidAssociatedAccount),
        }
    };
}

pub struct Processor {}
impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = TreasuryInstruction::try_from_slice(input)
            .map_err(|_| TreasuryError::InvalidInstruction)?;

        msg!("Instruction :: {:?}", instruction);

        match instruction {
            TreasuryInstruction::CreateSimpleTreasury { mode } => {
                Self::process_create_simple_treasury(program_id, accounts, mode)
            }
            TreasuryInstruction::WithdrawSimple { amount } => {
                Self::process_withdraw_simple(program_id, accounts, amount)
            }
            TreasuryInstruction::CreatedVestedTreaury {
                amount,
                period,
                percentage,
            } => Self::process_create_vested_treasury(
                program_id, accounts, amount, period, percentage,
            ),
            TreasuryInstruction::WithdrawVested => {
                Self::process_withdraw_vested(program_id, accounts)
            }
        }
    }

    // Mint::unpack(&token_info.data.borrow()).map_err(|_| TreasuryError::TokenNotSPLToken)?;

    pub fn process_create_simple_treasury(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        mode: SimpleTreasuryMode,
    ) -> ProgramResult {
        let iter = &mut accounts.iter();
        let funder_info = next_account_info(iter)?;
        let authority_info = next_account_info(iter)?;
        let treasury_info = next_account_info(iter)?;
        let mint_info = next_account_info(iter)?;
        let rent_info = next_account_info(iter)?;

        let rent = Rent::from_account_info(rent_info)?;

        Mint::unpack(&mint_info.data.borrow()).map_err(|_| TreasuryError::TokenNotSPLToken)?;

        // only allow creation of specific modes
        match mode {
            SimpleTreasuryMode::Locked => { /* ok */ }
            SimpleTreasuryMode::Unlocked => { /* ok */ }
        }

        if !treasury_info.data_is_empty() {
            return Err(TreasuryError::TreasuryAlreadyExists.into());
        }

        let user_treasury = SimpleTreasury {
            mint: *mint_info.key,
            mode,
            authority: *authority_info.key,
        };
        let data = user_treasury.try_to_vec()?;

        let lamports = rent.minimum_balance(data.len());
        let space = data.len() as u64;
        invoke(
            &system_instruction::create_account(
                funder_info.key,
                treasury_info.key,
                lamports,
                space,
                program_id,
            ),
            &[funder_info.clone(), treasury_info.clone()],
        )?;

        treasury_info.data.borrow_mut().copy_from_slice(&data);

        Ok(())
    }

    pub fn process_withdraw_simple(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let iter = &mut accounts.iter();
        let _funder_info = next_account_info(iter)?;
        let authority_info = next_account_info(iter)?;
        let recipient_info = next_account_info(iter)?;
        let treasury_info = next_account_info(iter)?;
        let fund_authority_info = next_account_info(iter)?;
        let fund_info = next_account_info(iter)?;
        let token_program_info = next_account_info(iter)?;

        let treasury =
            SimpleTreasury::from_account_info(treasury_info, authority_info, program_id)?;

        match treasury.mode {
            SimpleTreasuryMode::Locked => return Err(TreasuryError::TreasuryIsLocked.into()),
            SimpleTreasuryMode::Unlocked => { /* ok */ }
        }

        let fund_authority_seed = SimpleTreasury::verify_fund_authority_address(
            fund_authority_info.key,
            treasury_info.key,
            program_id,
        )?;

        if spl_associated_token_account::get_associated_token_address(
            fund_authority_info.key,
            &treasury.mint,
        ) != *fund_info.key
        {
            return Err(TreasuryError::InvalidTreasuryFundAddress.into());
        }

        verify_associated!(fund_info, *fund_authority_info.key, treasury.mint)?;
        verify_associated!(recipient_info, treasury.authority, treasury.mint)?;

        invoke_signed(
            // will fail if not enough funds
            &spl_token::instruction::transfer(
                &spl_token::id(),
                fund_info.key,
                recipient_info.key,
                fund_authority_info.key,
                &[],
                amount,
            )?,
            &[
                fund_info.clone(),
                recipient_info.clone(),
                treasury_info.clone(),
                token_program_info.clone(),
            ],
            &[&[
                b"simple authority",
                &treasury_info.key.to_bytes(),
                &[fund_authority_seed],
            ]],
        )
    }

    pub fn process_create_vested_treasury(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
        period: u64,
        percentage: u16,
    ) -> ProgramResult {
        let iter = &mut accounts.iter();
        let funder_info = next_account_info(iter)?;
        let authority_info = next_account_info(iter)?;
        let treasury_info = next_account_info(iter)?;
        let mint_info = next_account_info(iter)?;
        let rent_info = next_account_info(iter)?;
        let clock_info = next_account_info(iter)?;

        let rent = Rent::from_account_info(rent_info)?;
        let clock = Clock::from_account_info(clock_info)?;
        Mint::unpack(&mint_info.data.borrow()).map_err(|_| TreasuryError::TokenNotSPLToken)?;

        if amount == 0 {
            return Err(TreasuryError::InvalidVestmentAmount.into());
        }
        if period == 0 {
            return Err(TreasuryError::InvalidVestmentPeriod.into());
        }

        if percentage < VestedTreasury::MIN_PERCENTAGE
            || percentage > VestedTreasury::MAX_PERCENTAGE
        {
            return Err(TreasuryError::InvalidVestmentPercentage.into());
        }

        if !treasury_info.data_is_empty() {
            return Err(TreasuryError::TreasuryAlreadyExists.into());
        }

        let vested_treasury = VestedTreasury {
            mint: *mint_info.key,
            authority: *authority_info.key,
            initial_amount: amount,
            start: clock.unix_timestamp,
            vestment_period: period,
            vestment_percentage: percentage,
            withdrawn: 0,
        };
        let data = vested_treasury.try_to_vec()?;

        let lamports = rent.minimum_balance(data.len());
        let space = data.len() as u64;
        invoke(
            &system_instruction::create_account(
                funder_info.key,
                treasury_info.key,
                lamports,
                space,
                program_id,
            ),
            &[funder_info.clone(), treasury_info.clone()],
        )?;

        treasury_info.data.borrow_mut().copy_from_slice(&data);

        Ok(())
    }

    pub fn process_withdraw_vested(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let iter = &mut accounts.iter();
        let _funder_info = next_account_info(iter)?;
        let authority_info = next_account_info(iter)?;
        let recipient_info = next_account_info(iter)?;
        let treasury_info = next_account_info(iter)?;
        let fund_authority_info = next_account_info(iter)?;
        let fund_info = next_account_info(iter)?;
        let clock_info = next_account_info(iter)?;
        let token_program_info = next_account_info(iter)?;

        let clock = Clock::from_account_info(clock_info)?;

        let mut treasury =
            VestedTreasury::from_account_info(treasury_info, authority_info, program_id)?;

        let fund_authority_seed = VestedTreasury::verify_fund_authority_address(
            fund_authority_info.key,
            treasury_info.key,
            program_id,
        )?;

        if spl_associated_token_account::get_associated_token_address(
            fund_authority_info.key,
            &treasury.mint,
        ) != *fund_info.key
        {
            return Err(TreasuryError::InvalidTreasuryFundAddress.into());
        }
        let fund = verify_associated!(fund_info, *fund_authority_info.key, treasury.mint)?;
        verify_associated!(recipient_info, treasury.authority, treasury.mint)?;

        // calculate how much funds are available to be released
        let available = treasury.maximum_available(clock.unix_timestamp) - treasury.withdrawn;
        if available > 0 {
            let payable = if available > fund.amount {
                fund.amount
            } else {
                available
            };

            treasury.withdrawn += payable;
            treasury_info
                .data
                .borrow_mut()
                .copy_from_slice(&treasury.try_to_vec()?);

            invoke_signed(
                &spl_token::instruction::transfer(
                    &spl_token::id(),
                    fund_info.key,
                    recipient_info.key,
                    fund_authority_info.key,
                    &[],
                    payable,
                )?,
                &[
                    fund_authority_info.clone(),
                    fund_info.clone(),
                    recipient_info.clone(),
                    treasury_info.clone(),
                    token_program_info.clone(),
                ],
                &[&[
                    b"vested authority",
                    &treasury_info.key.to_bytes(),
                    &[fund_authority_seed],
                ]],
            )
        } else {
            Ok(())
        }
    }
}
