use std::u16;

use crate::error::TreasuryError;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::clock::UnixTimestamp;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct Settings {
    pub token: Pubkey,
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

    pub fn verify_mint(&self, key: &Pubkey) -> Result<(), ProgramError> {
        match self.token == *key {
            true => Ok(()),
            false => Err(TreasuryError::MintWrongToken.into()),
        }
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

    pub fn verify_program_address(
        key: &Pubkey,
        authority: &Pubkey,
        program_id: &Pubkey,
    ) -> Result<u8, ProgramError> {
        let (derived_key, seed) = Self::program_address(authority, program_id);
        if *key != derived_key {
            return Err(TreasuryError::InvalidTreasuryAddress.into());
        }
        Ok(seed)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct VestedTreasury {
    pub authority: Pubkey,
    pub initial_amount: u64,
    pub start: UnixTimestamp,
    pub vestment_period: u64,
    pub vestment_percentage: u16,
    pub withdrawn: u64,
}
impl VestedTreasury {
    pub const MIN_PERCENTAGE: u16 = 1;
    pub const MAX_PERCENTAGE: u16 = 10_000;

    pub fn program_address(authority: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"vested", &authority.to_bytes()], program_id)
    }

    pub fn verify_program_address(
        key: &Pubkey,
        authority: &Pubkey,
        program_id: &Pubkey,
    ) -> Result<u8, ProgramError> {
        let (derived_key, seed) = Self::program_address(authority, program_id);
        if *key != derived_key {
            return Err(TreasuryError::InvalidTreasuryAddress.into());
        }
        Ok(seed)
    }

    pub fn maximum_available(&self, now: UnixTimestamp) -> u64 {
        let period = now - self.start;
        if period <= 0 {
            return 0;
        }

        let ticks = period as u64 / self.vestment_period;
        let percentage = self.vestment_percentage as f64 / 10_000f64;
        let amount = (self.initial_amount as f64 * percentage) as u64 * ticks;
        if amount > self.initial_amount {
            self.initial_amount
        } else {
            amount
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_serialize_accounts() {
        let settings = Settings {
            token: Pubkey::new_unique(),
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

    #[test]
    pub fn test_vested_max() {
        let vest = VestedTreasury {
            authority: Pubkey::new_unique(),
            initial_amount: 100_000,
            start: 0,
            vestment_period: 60,
            vestment_percentage: 500, // 5%
            withdrawn: 0,
        };

        assert_eq!(vest.maximum_available(-5000), 0);
        assert_eq!(vest.maximum_available(0), 0);
        assert_eq!(vest.maximum_available(1), 0);
        assert_eq!(vest.maximum_available(59), 0);
        assert_eq!(vest.maximum_available(60), 5_000);
        assert_eq!(vest.maximum_available(61), 5_000);
        assert_eq!(vest.maximum_available(119), 5_000);
        assert_eq!(vest.maximum_available(120), 10_000);
        assert_eq!(vest.maximum_available(1_199), 95_000);
        assert_eq!(vest.maximum_available(1_200), 100_000);
        assert_eq!(vest.maximum_available(5_000), 100_000);
    }
}
