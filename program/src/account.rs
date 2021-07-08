use crate::error::TreasuryError;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
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

    pub fn verify_mint(&self, key: &Pubkey) -> Result<(), ProgramError> {
        match self.token == *key {
            true => Ok(()),
            false => Err(TreasuryError::MintWrongToken.into()),
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
#[derive(Clone, Copy, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum SimpleTreasuryMode {
    Locked,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct SimpleTreasury {
    pub mode: SimpleTreasuryMode,
    pub authority: Pubkey,
}

impl SimpleTreasury {
    pub fn program_address(authority: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"simple", &authority.to_bytes()], program_id)
    }

    pub fn verify_program_key(
        key: &Pubkey,
        owner: &Pubkey,
        program_id: &Pubkey,
    ) -> Result<u8, ProgramError> {
        let (derived_key, seed) = Self::program_address(owner, program_id);
        if *key != derived_key {
            return Err(TreasuryError::InvalidTreasuryAddress.into());
        }
        Ok(seed)
    }

    pub fn fund_address(treasury: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"simple fund", &treasury.to_bytes()], program_id)
    }

    pub fn verify_fund_address(
        key: &Pubkey,
        treasury: &Pubkey,
        program_id: &Pubkey,
    ) -> Result<u8, ProgramError> {
        let (derived_key, seed) = Self::fund_address(treasury, program_id);
        if *key != derived_key {
            return Err(TreasuryError::InvalidTreasuryFundAddress.into());
        }
        Ok(seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let user_treasury = SimpleTreasury {
            mode: SimpleTreasuryMode::Locked,
            authority: Pubkey::new_unique(),
        };
        let user_treasury_data = user_treasury.try_to_vec().unwrap();
        assert_eq!(
            user_treasury,
            SimpleTreasury::try_from_slice(&user_treasury_data).unwrap()
        );
    }
}
