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
            TreasuryInstruction::Initialize => Self::process_initialize(program_id, accounts),
            TreasuryInstruction::CreateSimpleTreasury => {
                Self::process_create_simple_treasury(program_id, accounts)
            }
        }
    }

    pub fn process_initialize(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let iter = &mut accounts.iter();
        let funder_info = next_account_info(iter)?;
        let token_info = next_account_info(iter)?;
        let authority_info = next_account_info(iter)?;
        let settings_info = next_account_info(iter)?;
        let rent_info = next_account_info(iter)?;
        let rent = Rent::from_account_info(rent_info)?;

        if settings_info.try_data_len()? > 0 {
            return Err(TreasuryError::AlreadyInitialized.into());
        }

        if !authority_info.is_signer {
            return Err(TreasuryError::MissingAuthoritySignature.into());
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
        let token_program_info = next_account_info(iter)?;

        let settings = Settings::try_from_slice(&settings_info.data.borrow())
            .map_err(|_| TreasuryError::NotInitialized)?;

        settings.verify_mint(mint_info.key)?;

        if !treasury_info.data_is_empty() {
            return Err(TreasuryError::UserTreasuryExists.into());
        }

        if !authority_info.is_signer {
            return Err(TreasuryError::MissingAuthoritySignature.into());
        }

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

        Ok(())
    }
}
