use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction::{self, create_account},
    sysvar::{rent::Rent, Sysvar},
};

use spl_token::state::{Account, Mint};

use crate::{
    account::{Settings, SimpleTreasury},
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
            TreasuryInstruction::Initialize {
                fee_user,
                fee_zoints,
            } => Self::process_initialize(program_id, accounts, fee_user, fee_zoints),
            TreasuryInstruction::CreateSimpleTreasury => {
                Self::process_create_simple_treasury(program_id, accounts)
            }
            TreasuryInstruction::UpdateFees {
                fee_user,
                fee_zoints,
            } => Self::process_update_fees(program_id, accounts, fee_user, fee_zoints),
        }
    }

    pub fn process_initialize(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        launch_fee_user: u64,
        launch_fee_zoints: u64,
    ) -> ProgramResult {
        let iter = &mut accounts.iter();
        let funder_info = next_account_info(iter)?;
        let token_info = next_account_info(iter)?;
        let authority_info = next_account_info(iter)?;
        let fee_recipient_info = next_account_info(iter)?;
        let settings_info = next_account_info(iter)?;
        let rent_info = next_account_info(iter)?;
        let rent = Rent::from_account_info(rent_info)?;

        if settings_info.try_data_len()? > 0 {
            return Err(TreasuryError::AlreadyInitialized.into());
        }

        if !authority_info.is_signer {
            return Err(TreasuryError::MissingAuthoritySignature.into());
        }

        let _ =
            Mint::unpack(&token_info.data.borrow()).map_err(|_| TreasuryError::TokenNotSPLToken)?;

        let fee_recipient = Account::unpack(&fee_recipient_info.data.borrow())
            .map_err(|_| TreasuryError::AssociatedAccountInvalid)?;
        if fee_recipient.mint != *token_info.key {
            return Err(TreasuryError::AssociatedAccountWrongMint.into());
        }

        let settings = Settings {
            token: *token_info.key,
            fee_recipient: *fee_recipient_info.key,
            price_authority: *authority_info.key,
            launch_fee_user,
            launch_fee_zoints,
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

    pub fn process_update_fees(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        fee_user: u64,
        fee_zoints: u64,
    ) -> ProgramResult {
        let iter = &mut accounts.iter();
        let _funder_info = next_account_info(iter)?;
        let authority_info = next_account_info(iter)?;
        let fee_recipient_info = next_account_info(iter)?;
        let settings_info = next_account_info(iter)?;

        if settings_info.data_len() == 0 {
            return Err(TreasuryError::NotInitialized.into());
        }

        let mut settings = Settings::try_from_slice(&settings_info.data.borrow())?;
        settings.verify_price_authority(authority_info)?;

        let fee_recipient = Account::unpack(&fee_recipient_info.data.borrow())
            .map_err(|_| TreasuryError::AssociatedAccountInvalid)?;
        if fee_recipient.mint != settings.token {
            return Err(TreasuryError::AssociatedAccountWrongMint.into());
        }

        settings.fee_recipient = *fee_recipient_info.key;
        settings.launch_fee_user = fee_user;
        settings.launch_fee_zoints = fee_zoints;

        settings_info
            .data
            .borrow_mut()
            .copy_from_slice(&settings.try_to_vec()?);

        Ok(())
    }

    pub fn process_create_simple_treasury(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let iter = &mut accounts.iter();
        let funder_info = next_account_info(iter)?;
        let authority_info = next_account_info(iter)?;
        let authority_associated_info = next_account_info(iter)?;
        let treasury_info = next_account_info(iter)?;
        let treasury_fund_info = next_account_info(iter)?;
        let mint_info = next_account_info(iter)?;
        let settings_info = next_account_info(iter)?;
        let fee_recipient_info = next_account_info(iter)?;
        let rent_info = next_account_info(iter)?;

        let rent = Rent::from_account_info(rent_info)?;
        let token_program_info = next_account_info(iter)?;

        let settings = Settings::try_from_slice(&settings_info.data.borrow())
            .map_err(|_| TreasuryError::NotInitialized)?;

        settings.verify_mint(mint_info.key)?;
        settings.verify_fee_recipient(fee_recipient_info.key)?;

        if !treasury_info.data_is_empty() {
            return Err(TreasuryError::UserTreasuryExists.into());
        }

        if !authority_info.is_signer {
            return Err(TreasuryError::MissingAuthoritySignature.into());
        }

        let (_mint, _associated_account) = settings.verify_token_and_fee_payer(
            mint_info,
            authority_info,
            authority_associated_info,
            settings.launch_fee_user,
        )?;

        let treasury_seed =
            SimpleTreasury::verify_program_key(treasury_info.key, authority_info.key, program_id)?;

        let fund_seed = SimpleTreasury::verify_fund_address(
            treasury_fund_info.key,
            treasury_info.key,
            program_id,
        )?;

        if treasury_fund_info.data_is_empty() {
            let lamports = rent.minimum_balance(Account::LEN);
            let space = Account::LEN as u64;
            invoke_signed(
                &create_account(
                    funder_info.key,
                    treasury_fund_info.key,
                    lamports,
                    space,
                    &spl_token::id(),
                ),
                &[funder_info.clone(), treasury_fund_info.clone()],
                &[&[b"simple fund", &treasury_info.key.to_bytes(), &[fund_seed]]],
            )?;

            invoke(
                &spl_token::instruction::initialize_account(
                    &spl_token::id(),
                    treasury_fund_info.key,
                    mint_info.key,
                    treasury_info.key,
                )?,
                &[
                    treasury_fund_info.clone(),
                    mint_info.clone(),
                    rent_info.clone(),
                    treasury_info.clone(),
                    token_program_info.clone(),
                ],
            )?;

            msg!("treasury fund account created");
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
            &[&[b"user", &authority_info.key.to_bytes(), &[treasury_seed]]],
        )?;

        treasury_info.data.borrow_mut().copy_from_slice(&data);

        invoke(
            &spl_token::instruction::transfer(
                &spl_token::id(),
                authority_associated_info.key,
                fee_recipient_info.key,
                authority_info.key,
                &[],
                settings.launch_fee_user,
            )?,
            &[
                authority_associated_info.clone(),
                fee_recipient_info.clone(),
                authority_info.clone(),
                token_program_info.clone(),
            ],
        )
    }
}
