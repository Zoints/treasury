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

use crate::{
    account::{Settings, ZointsCommunity},
    error::TreasuryError,
    instruction::TreasuryInstruction,
};

pub struct Processor {}
impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        match TreasuryInstruction::unpack(input)? {
            TreasuryInstruction::Initialize { token } => {
                msg!("Instruction :: Initialize");
                Self::process_initialize(program_id, accounts, token)
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

    pub fn process_initialize(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        token: Pubkey,
    ) -> ProgramResult {
        let iter = &mut accounts.iter();
        let funder = next_account_info(iter)?;
        let token_info = next_account_info(iter)?;
        let settings_info = next_account_info(iter)?;
        let rent_info = next_account_info(iter)?;
        let rent = Rent::from_account_info(rent_info)?;

        // verify that token is an spl mint

        Settings::verify_program_key(settings_info.key, program_id)?;

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
