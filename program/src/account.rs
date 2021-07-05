use crate::error::TreasuryError;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::pubkey::MAX_SEED_LEN;
use spl_token::state::{Account as SPLAccount, Mint};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct Settings {
    pub token: Pubkey,
    pub fee_recipient: Pubkey,
    pub price_authority: Pubkey,
    pub launch_fee_user: u64,
    pub launch_fee_zoints: u64,
}

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
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct UserTreasury {
    pub authority: Pubkey,
}

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

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct ZointsTreasury {
    pub authority: Pubkey,
}

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

    #[test]
    pub fn test_serialize_accounts() {
        let settings = Settings {
            token: Pubkey::new_unique(),
            fee_recipient: Pubkey::new_unique(),
            price_authority: Pubkey::new_unique(),
            launch_fee_user: 9238478234,
            launch_fee_zoints: 1239718515,
        };
        let settings_data = settings.try_to_vec().unwrap();
        assert_eq!(settings, Settings::try_from_slice(&settings_data).unwrap());

        let user_treasury = UserTreasury {
            authority: Pubkey::new_unique(),
        };
        let user_treasury_data = user_treasury.try_to_vec().unwrap();
        assert_eq!(
            user_treasury,
            UserTreasury::try_from_slice(&user_treasury_data).unwrap()
        );

        let zoints_treasury = ZointsTreasury {
            authority: Pubkey::new_unique(),
        };
        let zoints_treasury_data = zoints_treasury.try_to_vec().unwrap();
        assert_eq!(
            zoints_treasury,
            ZointsTreasury::try_from_slice(&zoints_treasury_data).unwrap()
        );
    }
}
