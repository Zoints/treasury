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

use spl_token::state::Mint;

use crate::{
    account::{Settings, SimpleTreasury, SimpleTreasuryMode, VestedTreasury},
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
        mode: SimpleTreasuryMode,
    ) -> ProgramResult {
        let iter = &mut accounts.iter();
        let funder_info = next_account_info(iter)?;
        let authority_info = next_account_info(iter)?;
        let treasury_info = next_account_info(iter)?;
        let rent_info = next_account_info(iter)?;

        let rent = Rent::from_account_info(rent_info)?;

        // only allow creation of specific modes
        match mode {
            SimpleTreasuryMode::Locked => { /* ok */ }
            SimpleTreasuryMode::Unlocked => { /* ok */ }
        }

        if !treasury_info.data_is_empty() {
            return Err(TreasuryError::TreasuryAlreadyExists.into());
        }

        let user_treasury = SimpleTreasury {
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
        let mint_info = next_account_info(iter)?;
        let settings_info = next_account_info(iter)?;
        let token_program_info = next_account_info(iter)?;

        Settings::verify_program_key(settings_info.key, program_id)?;
        let settings = Settings::try_from_slice(&settings_info.data.borrow())
            .map_err(|_| TreasuryError::NotInitialized)?;
        settings.verify_mint(mint_info.key)?;

        if !authority_info.is_signer {
            return Err(TreasuryError::MissingAuthoritySignature.into());
        }

        if *treasury_info.owner != *program_id {
            return Err(TreasuryError::InvalidTreasuryOwner.into());
        }

        let treasury = SimpleTreasury::try_from_slice(&treasury_info.data.borrow())
            .map_err(|_| TreasuryError::InvalidTreasuryAddress)?;

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
            mint_info.key,
        ) != *fund_info.key
        {
            return Err(TreasuryError::InvalidTreasuryFundAddress.into());
        }
        //let fund = spl_token::state::Account::unpack(&fund_info.data.borrow())
        //    .map_err(|_| TreasuryError::InvalidTreasuryFundAccount)?;

        let recipient = spl_token::state::Account::unpack(&recipient_info.data.borrow())
            .map_err(|_| TreasuryError::InvalidRecipientAccount)?;
        if recipient.owner != *authority_info.key {
            return Err(TreasuryError::InvalidRecipient.into());
        }
        if recipient.mint != *mint_info.key {
            return Err(TreasuryError::MintInvalid.into());
        }

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
                //funder_info.clone(),
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

        if percentage < VestedTreasury::MIN_PERCENTAGE
            || percentage > VestedTreasury::MAX_PERCENTAGE
        {
            return Err(TreasuryError::InvalidVestmentPercentage.into());
        }

        if !treasury_info.data_is_empty() {
            return Err(TreasuryError::TreasuryAlreadyExists.into());
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
        let mint_info = next_account_info(iter)?;
        let settings_info = next_account_info(iter)?;
        let clock_info = next_account_info(iter)?;
        let token_program_info = next_account_info(iter)?;

        let clock = Clock::from_account_info(clock_info)?;

        Settings::verify_program_key(settings_info.key, program_id)?;
        let settings = Settings::try_from_slice(&settings_info.data.borrow())
            .map_err(|_| TreasuryError::NotInitialized)?;
        settings.verify_mint(mint_info.key)?;

        if !authority_info.is_signer {
            return Err(TreasuryError::MissingAuthoritySignature.into());
        }

        if *treasury_info.owner != *program_id {
            return Err(TreasuryError::InvalidTreasuryOwner.into());
        }

        let mut treasury = VestedTreasury::try_from_slice(&treasury_info.data.borrow())
            .map_err(|_| TreasuryError::InvalidTreasuryAddress)?;

        let fund_authority_seed = VestedTreasury::verify_fund_authority_address(
            fund_authority_info.key,
            treasury_info.key,
            program_id,
        )?;

        if spl_associated_token_account::get_associated_token_address(
            fund_authority_info.key,
            mint_info.key,
        ) != *fund_info.key
        {
            return Err(TreasuryError::InvalidTreasuryFundAddress.into());
        }
        let fund = spl_token::state::Account::unpack(&fund_info.data.borrow())
            .map_err(|_| TreasuryError::InvalidTreasuryFundAccount)?;

        let recipient = spl_token::state::Account::unpack(&recipient_info.data.borrow())
            .map_err(|_| TreasuryError::InvalidRecipientAccount)?;
        if recipient.owner != *authority_info.key {
            return Err(TreasuryError::InvalidRecipient.into());
        }
        if recipient.mint != *mint_info.key {
            return Err(TreasuryError::MintInvalid.into());
        }

        // calculate how much funds are available to be released
        let available = treasury.maximum_available(clock.unix_timestamp) - treasury.withdrawn;
        if available > 0 {
            let payable = if available > fund.amount {
                fund.amount
            } else {
                available
            };

            treasury.withdrawn += available;
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
                    //funder_info.clone(),
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
