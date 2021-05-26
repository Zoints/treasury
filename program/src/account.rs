use crate::error::TreasuryError;
use arrayref::{array_mut_ref, array_ref};
use arrayref::{array_refs, mut_array_refs};
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
    pub fee_recipient: Pubkey,
    pub price_authority: Pubkey,
    pub launch_fee_user: u64,
    pub launch_fee_zoints: u64,
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
        let seed = Settings::verify_program_key(settings_info.key, program_id)?;
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
    const LEN: usize = 32 + 32 + 32 + 8 + 8;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, Settings::LEN];
        let (token, fee_recipient, price_authority, launch_fee_user, launch_fee_zoints) =
            array_refs![src, 32, 32, 32, 8, 8];
        let token = Pubkey::new(token);
        let fee_recipient = Pubkey::new(fee_recipient);
        let price_authority = Pubkey::new(price_authority);
        let launch_fee_user = u64::from_le_bytes(*launch_fee_user);
        let launch_fee_zoints = u64::from_le_bytes(*launch_fee_zoints);
        Ok(Settings {
            token,
            fee_recipient,
            price_authority,
            launch_fee_user,
            launch_fee_zoints,
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Settings::LEN];
        let (
            token_dst,
            fee_recipient_dst,
            price_authority_dst,
            launch_fee_user_dst,
            launch_fee_zoints_dst,
        ) = mut_array_refs![dst, 32, 32, 32, 8, 8];
        *token_dst = self.token.to_bytes();
        *fee_recipient_dst = self.fee_recipient.to_bytes();
        *price_authority_dst = self.price_authority.to_bytes();
        *launch_fee_user_dst = self.launch_fee_user.to_le_bytes();
        *launch_fee_zoints_dst = self.launch_fee_zoints.to_le_bytes();
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UserTreasury {
    pub authority: Pubkey,
}

impl Sealed for UserTreasury {}

impl UserTreasury {
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
            return Err(TreasuryError::InvalidUserTreasuryKey.into());
        }
        Ok(seed)
    }
}

impl Pack for UserTreasury {
    const LEN: usize = 32;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, UserTreasury::LEN];

        let authority = Pubkey::new(src);

        Ok(UserTreasury { authority })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let authority_dst = array_mut_ref![dst, 0, UserTreasury::LEN];
        *authority_dst = self.authority.to_bytes();
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ZointsTreasury {
    pub authority: Pubkey,
}

impl Sealed for ZointsTreasury {}

impl ZointsTreasury {
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
            return Err(TreasuryError::InvalidZointsTreasuryKey.into());
        }
        Ok(seed)
    }

    fn valid_character(n: u8) -> bool {
        match n as char {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '-' | '.' | '(' | ')' => true,
            _ => false,
        }
    }

    pub fn valid_name(name: &Vec<u8>) -> Result<(), ProgramError> {
        if name.len() < 1 {
            return Err(TreasuryError::ZointsTreasuryNameTooShort.into());
        }
        if name.len() > MAX_SEED_LEN {
            return Err(TreasuryError::ZointsTreasuryNameTooLong.into());
        }

        match name.iter().all(|&n| ZointsTreasury::valid_character(n)) {
            true => Ok(()),
            false => Err(TreasuryError::ZointsTreasuryNameInvalidCharacters.into()),
        }
    }
}

impl Pack for ZointsTreasury {
    const LEN: usize = 32;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, ZointsTreasury::LEN];

        let authority = Pubkey::new(src);

        Ok(ZointsTreasury { authority })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let authority_dst = array_mut_ref![dst, 0, ZointsTreasury::LEN];
        *authority_dst = self.authority.to_bytes();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cast_error(e: TreasuryError) -> Result<(), ProgramError> {
        return Err(e.into());
    }

    #[test]
    pub fn test_verify_name() {
        assert_eq!(
            ZointsTreasury::valid_name(&b"".to_vec()),
            cast_error(TreasuryError::ZointsTreasuryNameTooShort)
        );
        assert_eq!(
            ZointsTreasury::valid_name(&b"000000000000000000000000000000001".to_vec()),
            cast_error(TreasuryError::ZointsTreasuryNameTooLong)
        );
        assert_eq!(ZointsTreasury::valid_name(&b"a".to_vec()), Ok(()));

        assert_eq!(
            ZointsTreasury::valid_name(&b"00000000000000000000000000000000".to_vec()),
            Ok(())
        );
        assert_eq!(ZointsTreasury::valid_name(&b"valid_name".to_vec()), Ok(()));
        assert_eq!(ZointsTreasury::valid_name(&b"aAzZ09-_.()".to_vec()), Ok(()));
        assert_eq!(
            ZointsTreasury::valid_name(&b"invalid name".to_vec()),
            cast_error(TreasuryError::ZointsTreasuryNameInvalidCharacters)
        );
        assert_eq!(
            ZointsTreasury::valid_name(&b"%".to_vec()),
            cast_error(TreasuryError::ZointsTreasuryNameInvalidCharacters)
        );
        assert_eq!(
            ZointsTreasury::valid_name(&b"%20".to_vec()),
            cast_error(TreasuryError::ZointsTreasuryNameInvalidCharacters)
        );
        assert_eq!(
            ZointsTreasury::valid_name(&b"random{word".to_vec()),
            cast_error(TreasuryError::ZointsTreasuryNameInvalidCharacters)
        );
    }
}
