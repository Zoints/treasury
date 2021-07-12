use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction::{self},
    sysvar::{rent::Rent, Sysvar},
};

use spl_token::state::Mint;

use crate::{
    account::{Settings, SimpleTreasury, VestedTreasury},
    error::TreasuryError,
    instruction::TreasuryInstruction,
};

pub struct Processor {}
impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = TreasuryInstruction::try_from_slice(input)
            .map_err(|_| TreasuryError::InvalidInstruction)?;

        msg!("Instruction :: {:?}", instruction);

        match instruction {
            TreasuryInstruction::Initialize => Self::process_initialize(program_id, accounts),
            TreasuryInstruction::CreateSimpleTreasury => {
                Self::process_create_simple_treasury(program_id, accounts)
            }
            TreasuryInstruction::CreatedVestedTreaury {
                amount,
                period,
                percentage,
            } => Self::process_create_vested_treasury(
                program_id, accounts, amount, period, percentage,
            ),
        }
    }

    pub fn process_initialize(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let iter = &mut accounts.iter();
        let funder_info = next_account_info(iter)?;
        let token_info = next_account_info(iter)?;
        let settings_info = next_account_info(iter)?;
        let rent_info = next_account_info(iter)?;
        let rent = Rent::from_account_info(rent_info)?;

        if settings_info.try_data_len()? > 0 {
            return Err(TreasuryError::AlreadyInitialized.into());
        }

        Mint::unpack(&token_info.data.borrow()).map_err(|_| TreasuryError::TokenNotSPLToken)?;

        let settings = Settings {
            token: *token_info.key,
        };

        let data = settings.try_to_vec()?;

        let seed = Settings::verify_program_key(settings_info.key, program_id)?;
        let lamports = rent.minimum_balance(data.len());
        let space = data.len() as u64;
        invoke_signed(
            &system_instruction::create_account(
                funder_info.key,
                settings_info.key,
                lamports,
                space,
                program_id,
            ),
            &[funder_info.clone(), settings_info.clone()],
            &[&[b"settings", &[seed]]],
        )?;

        settings_info.data.borrow_mut().copy_from_slice(&data);

        Ok(())
    }

    pub fn process_create_simple_treasury(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let iter = &mut accounts.iter();
        let funder_info = next_account_info(iter)?;
        let authority_info = next_account_info(iter)?;
        let treasury_info = next_account_info(iter)?;
        let treasury_fund_info = next_account_info(iter)?;
        let mint_info = next_account_info(iter)?;
        let settings_info = next_account_info(iter)?;
        let rent_info = next_account_info(iter)?;

        let rent = Rent::from_account_info(rent_info)?;

        let settings = Settings::try_from_slice(&settings_info.data.borrow())
            .map_err(|_| TreasuryError::NotInitialized)?;

        settings.verify_mint(mint_info.key)?;

        if !treasury_info.data_is_empty() {
            return Err(TreasuryError::TreasuryAlreadyExists.into());
        }

        if !authority_info.is_signer {
            return Err(TreasuryError::MissingAuthoritySignature.into());
        }

        let treasury_seed = SimpleTreasury::verify_program_address(
            treasury_info.key,
            authority_info.key,
            program_id,
        )?;

        let fund_address = spl_associated_token_account::get_associated_token_address(
            treasury_info.key,
            mint_info.key,
        );

        if fund_address != *treasury_fund_info.key {
            return Err(TreasuryError::InvalidTreasuryFundAddress.into());
        }

        let user_treasury = SimpleTreasury {
            mode: crate::account::SimpleTreasuryMode::Locked,
            authority: *authority_info.key,
        };
        let data = user_treasury.try_to_vec()?;

        let lamports = rent.minimum_balance(data.len());
        let space = data.len() as u64;
        invoke_signed(
            &system_instruction::create_account(
                funder_info.key,
                treasury_info.key,
                lamports,
                space,
                program_id,
            ),
            &[funder_info.clone(), treasury_info.clone()],
            &[&[b"simple", &authority_info.key.to_bytes(), &[treasury_seed]]],
        )?;

        treasury_info.data.borrow_mut().copy_from_slice(&data);

        Ok(())
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
        let treasury_fund_info = next_account_info(iter)?;
        let mint_info = next_account_info(iter)?;
        let settings_info = next_account_info(iter)?;
        let rent_info = next_account_info(iter)?;
        let clock_info = next_account_info(iter)?;

        let rent = Rent::from_account_info(rent_info)?;
        let clock = Clock::from_account_info(clock_info)?;

        if amount == 0 {
            return Err(TreasuryError::InvalidVestmentAmount.into());
        }
        if period == 0 {
            return Err(TreasuryError::InvalidVestmentPeriod.into());
        }

        if percentage < 1 || percentage > 1000 {
            return Err(TreasuryError::InvalidVestmentPercentage.into());
        }

        let settings = Settings::try_from_slice(&settings_info.data.borrow())
            .map_err(|_| TreasuryError::NotInitialized)?;
        settings.verify_mint(mint_info.key)?;

        if !treasury_info.data_is_empty() {
            return Err(TreasuryError::TreasuryAlreadyExists.into());
        }

        if !authority_info.is_signer {
            return Err(TreasuryError::MissingAuthoritySignature.into());
        }

        let treasury_seed = VestedTreasury::verify_program_address(
            treasury_info.key,
            authority_info.key,
            program_id,
        )?;

        let fund_address = spl_associated_token_account::get_associated_token_address(
            treasury_info.key,
            mint_info.key,
        );

        if fund_address != *treasury_fund_info.key {
            return Err(TreasuryError::InvalidTreasuryFundAddress.into());
        }

        let vested_treasury = VestedTreasury {
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
        invoke_signed(
            &system_instruction::create_account(
                funder_info.key,
                treasury_info.key,
                lamports,
                space,
                program_id,
            ),
            &[funder_info.clone(), treasury_info.clone()],
            &[&[b"vested", &authority_info.key.to_bytes(), &[treasury_seed]]],
        )?;

        treasury_info.data.borrow_mut().copy_from_slice(&data);

        Ok(())
    }
}
