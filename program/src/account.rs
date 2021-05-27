use crate::error::TreasuryError;
use arrayref::{array_mut_ref, array_ref};
use arrayref::{array_refs, mut_array_refs};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::pubkey::MAX_SEED_LEN;
use solana_program::system_instruction;
use solana_program::{
    msg,
    program_pack::{Pack, Sealed},
};
use spl_token::state::{Account as SPLAccount, Mint};

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

    pub fn verify_fee_recipient(&self, key: &Pubkey) -> Result<(), ProgramError> {
        match self.fee_recipient == *key {
            true => Ok(()),
            false => Err(TreasuryError::InvalidFeeRecipient.into()),
        }
    }

    pub fn verify_price_authority(
        &self,
        price_authority_info: &AccountInfo,
    ) -> Result<(), ProgramError> {
        if !price_authority_info.is_signer {
            return Err(TreasuryError::MissingAuthoritySignature.into());
        }

        match self.price_authority == *price_authority_info.key {
            true => Ok(()),
            false => Err(TreasuryError::InvalidPriceAuthority.into()),
        }
    }

    pub fn verify_token_and_fee_payer(
        &self,
        mint_info: &AccountInfo,
        owner: &AccountInfo,
        associated_account: &AccountInfo,
        fee: u64,
    ) -> Result<(Mint, SPLAccount), ProgramError> {
        let mint =
            Mint::unpack(&mint_info.data.borrow()).map_err(|_| TreasuryError::MintInvalid)?;

        let associated_account = SPLAccount::unpack(&associated_account.data.borrow())?;
        if associated_account.owner != *owner.key {
            return Err(TreasuryError::AssociatedAccountInvalid.into());
        }

        if associated_account.mint != *mint_info.key {
            return Err(TreasuryError::AssociatedAccountWrongMint.into());
        }

        if fee > 0 && associated_account.amount < fee {
            return Err(TreasuryError::NotEnoughZEE.into());
        }

        Ok((mint, associated_account))
    }

    pub fn verify_treasury_associated_account(
        &self,
        treasury_info: &AccountInfo,
        associated_info: &AccountInfo,
    ) -> Result<(), ProgramError> {
        let derived = spl_associated_token_account::get_associated_token_address(
            treasury_info.key,
            &self.token,
        );

        if derived != *associated_info.key {
            return Err(TreasuryError::TreasuryAssociatedAccountInvalid.into());
        }

        Ok(())
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

    pub fn create_account<'a>(
        funder_info: &AccountInfo<'a>,
        treasury_info: &AccountInfo<'a>,
        treasury_associated_info: &AccountInfo<'a>,
        mint_info: &AccountInfo<'a>,
        creator_info: &AccountInfo<'a>,
        rent_info: &AccountInfo<'a>,
        token_program_info: &AccountInfo<'a>,
        rent: solana_program::rent::Rent,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let seed =
            UserTreasury::verify_program_key(treasury_info.key, creator_info.key, program_id)?;
        let lamports = rent.minimum_balance(UserTreasury::LEN);
        let space = UserTreasury::LEN as u64;
        invoke_signed(
            &system_instruction::create_account(
                funder_info.key,
                treasury_info.key,
                lamports,
                space,
                program_id,
            ),
            &[funder_info.clone(), treasury_info.clone()],
            &[&[b"user", &creator_info.key.to_bytes(), &[seed]]],
        )?;
        invoke(
            &spl_associated_token_account::create_associated_token_account(
                funder_info.key,
                treasury_info.key,
                mint_info.key,
            ),
            &[
                funder_info.clone(),
                treasury_info.clone(),
                treasury_associated_info.clone(),
                mint_info.clone(),
                rent_info.clone(),
                token_program_info.clone(),
            ],
        )
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

    pub fn create_account<'a>(
        funder_info: &AccountInfo<'a>,
        treasury_info: &AccountInfo<'a>,
        name: &Vec<u8>,
        rent: solana_program::rent::Rent,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let seed = ZointsTreasury::verify_program_key(treasury_info.key, name, program_id)?;
        let lamports = rent.minimum_balance(ZointsTreasury::LEN);
        let space = ZointsTreasury::LEN as u64;
        invoke_signed(
            &system_instruction::create_account(
                funder_info.key,
                treasury_info.key,
                lamports,
                space,
                program_id,
            ),
            &[funder_info.clone(), treasury_info.clone()],
            &[&[b"zoints", name, &[seed]]],
        )
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
