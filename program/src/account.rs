use crate::error::TreasuryError;
use arrayref::{array_mut_ref, array_ref};
use regex::bytes::Regex;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::pubkey::MAX_SEED_LEN;
use solana_program::{
    msg,
    program_pack::{Pack, Sealed},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UserCommunity {
    authority: Pubkey,
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
    authority: Pubkey,
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
        match Regex::new(r"^[A-Za-z0-9_\-]+$").unwrap().is_match(name) {
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
