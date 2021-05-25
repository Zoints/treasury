use arrayref::{array_ref, array_refs};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_option::COption,
    program_pack::{IsInitialized, Pack},
    pubkey::{Pubkey, MAX_SEED_LEN},
    system_instruction,
    sysvar::{self, clock::Clock, rent::Rent, slot_hashes::SlotHashes, Sysvar, SysvarId},
};

use spl_token::state::Mint;

use crate::{
    account::{Settings, ZointsCommunity},
    error::TreasuryError,
    instruction::TreasuryInstruction,
};

pub struct Processor {}
impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        match TreasuryInstruction::unpack(input)? {
            TreasuryInstruction::Initialize => {
                msg!("Instruction :: Initialize");
                Self::process_initialize(program_id, accounts)
            }
            TreasuryInstruction::CreateUserTreasury => {
                msg!("Instruction :: CreateUserTreasury");
                Self::process_create_user_treasury(program_id, accounts)
            }
            TreasuryInstruction::CreateCommunityTreasury { name } => {
                msg!("Instruction :: CreateCommunityTreasury");
                Self::process_create_zoints_treasury(program_id, accounts, name)
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

        let _ = Mint::unpack(&token_info.data.borrow_mut())
            .map_err(|_| TreasuryError::TokenNotSPLToken)?;

        Settings::verify_program_key(settings_info.key, program_id)?;

        Settings::create_account(funder_info, settings_info, rent, program_id)?;

        let settings = Settings {
            token: *token_info.key,
        };

        Settings::pack(settings, &mut settings_info.data.borrow_mut())?;

        Ok(())
    }

    pub fn process_create_user_treasury(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let iter = &mut accounts.iter();
        let creator = next_account_info(iter)?;

        Ok(())
    }
    pub fn process_create_zoints_treasury(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        name: Vec<u8>,
    ) -> ProgramResult {
        ZointsCommunity::valid_name(&name)?;

        Ok(())
    }
}
