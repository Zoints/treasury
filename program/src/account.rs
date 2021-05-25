use crate::error::TreasuryError;
use arrayref::{array_mut_ref, array_ref};
use regex::bytes::Regex;
use solana_program::account_info::Account;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::pubkey::MAX_SEED_LEN;
use solana_program::system_instruction;
use solana_program::{
    msg,
    program_pack::{Pack, Sealed},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Settings {
    pub token: Pubkey,
}

impl Sealed for Settings {}

impl Settings {
    pub fn program_address(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"settings"], program_id)
    }

    pub fn verify_program_key(key: &Pubkey, program_id: &Pubkey) -> Result<u8, ProgramError> {
        let (derived_key, seed) = Self::program_address(program_id);
        if *key != derived_key {
            msg!("invalid settings account");
            return Err(TreasuryError::InvalidSettingsKey.into());
        }
        Ok(seed)
    }

    pub fn create_account<'a>(
        funder_info: &AccountInfo<'a>,
        settings_info: &AccountInfo<'a>,
        rent: solana_program::rent::Rent,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let (_, seed) = Settings::program_address(program_id);

        let lamports = rent.minimum_balance(Settings::LEN);
        let space = Settings::LEN as u64;
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
        )
    }
}

impl Pack for Settings {
    const LEN: usize = 32;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, Settings::LEN];
        let token = Pubkey::new(src);
        Ok(Settings { token })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let token_dst = array_mut_ref![dst, 0, Settings::LEN];
        *token_dst = self.token.to_bytes();
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UserCommunity {
    pub authority: Pubkey,
}

impl Sealed for UserCommunity {}

impl UserCommunity {
    pub fn program_address(user: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"user", &user.to_bytes()], program_id)
    }

    pub fn verify_program_key(
        key: &Pubkey,
        user: &Pubkey,
        program_id: &Pubkey,
    ) -> Result<u8, ProgramError> {
        let (derived_key, seed) = Self::program_address(user, program_id);
        if *key != derived_key {
            msg!("invalid user community account");
            return Err(TreasuryError::InvalidUserCommunityKey.into());
        }
        Ok(seed)
    }
}

impl Pack for UserCommunity {
    const LEN: usize = 32;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, UserCommunity::LEN];

        let authority = Pubkey::new(src);

        Ok(UserCommunity { authority })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let authority_dst = array_mut_ref![dst, 0, UserCommunity::LEN];
        *authority_dst = self.authority.to_bytes();
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ZointsCommunity {
    pub authority: Pubkey,
}

impl Sealed for ZointsCommunity {}

impl ZointsCommunity {
    pub fn program_address(name: &Vec<u8>, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"zoints", name], program_id)
    }

    pub fn verify_program_key(
        key: &Pubkey,
        name: &Vec<u8>,
        program_id: &Pubkey,
    ) -> Result<u8, ProgramError> {
        let (derived_key, seed) = Self::program_address(name, program_id);
        if *key != derived_key {
            msg!("invalid zoints community account");
            return Err(TreasuryError::InvalidZointsCommunityKey.into());
        }
        Ok(seed)
    }

    pub fn valid_name(name: &Vec<u8>) -> Result<(), ProgramError> {
        if name.len() < 1 {
            return Err(TreasuryError::ZointsCommunityNameTooShort.into());
        }
        if name.len() > MAX_SEED_LEN {
            return Err(TreasuryError::ZointsCommunityNameTooLong.into());
        }
        match Regex::new(r"^[A-Za-z0-9_\-\.()]+$").unwrap().is_match(name) {
            true => Ok(()),
            false => Err(TreasuryError::ZointsCommunityNameTooShort.into()),
        }
    }
}

impl Pack for ZointsCommunity {
    const LEN: usize = 32;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, ZointsCommunity::LEN];

        let authority = Pubkey::new(src);

        Ok(ZointsCommunity { authority })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let authority_dst = array_mut_ref![dst, 0, ZointsCommunity::LEN];
        *authority_dst = self.authority.to_bytes();
    }
}
