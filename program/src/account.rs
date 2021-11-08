use crate::error::TreasuryError;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, clock::UnixTimestamp, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum SimpleTreasuryMode {
    Locked,
    Unlocked,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct SimpleTreasury {
    pub mint: Pubkey,
    pub mode: SimpleTreasuryMode,
    pub authority: Pubkey,
}

impl SimpleTreasury {
    pub fn from_account_info(
        treasury_info: &AccountInfo,
        authority_info: &AccountInfo,
        program_id: &Pubkey,
    ) -> Result<SimpleTreasury, ProgramError> {
        // treasury account checks
        if *treasury_info.owner != *program_id {
            msg!("treasury account not owned by program");
            return Err(TreasuryError::InvalidTreasuryFundAccount.into());
        }
        let treasury = Self::try_from_slice(&treasury_info.data.borrow())
            .map_err(|_| TreasuryError::InvalidTreasuryFundAccount)?;

        // authority owner checks
        if !authority_info.is_signer {
            return Err(TreasuryError::MissingAuthoritySignature.into());
        }

        if treasury.authority != *authority_info.key {
            return Err(TreasuryError::InvalidTreasuryOwner.into());
        }

        Ok(treasury)
    }

    pub fn fund_authority_address(treasury_id: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"simple authority", &treasury_id.to_bytes()], program_id)
    }

    pub fn verify_fund_authority_address(
        key: &Pubkey,
        authority: &Pubkey,
        program_id: &Pubkey,
    ) -> Result<u8, ProgramError> {
        let (derived_key, seed) = Self::fund_authority_address(authority, program_id);
        if *key != derived_key {
            return Err(TreasuryError::InvalidTreasuryFundAuthorityAddress.into());
        }
        Ok(seed)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct VestedTreasury {
    pub mint: Pubkey,
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

    pub fn from_account_info(
        treasury_info: &AccountInfo,
        authority_info: &AccountInfo,
        program_id: &Pubkey,
    ) -> Result<VestedTreasury, ProgramError> {
        // treasury account checks
        if *treasury_info.owner != *program_id {
            msg!("treasury account not owned by program");
            return Err(TreasuryError::InvalidTreasuryFundAccount.into());
        }
        let treasury = Self::try_from_slice(&treasury_info.data.borrow())
            .map_err(|_| TreasuryError::InvalidTreasuryFundAccount)?;

        // authority owner checks
        if !authority_info.is_signer {
            return Err(TreasuryError::MissingAuthoritySignature.into());
        }

        if treasury.authority != *authority_info.key {
            return Err(TreasuryError::InvalidTreasuryOwner.into());
        }

        Ok(treasury)
    }

    pub fn fund_authority_address(treasury_id: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"vested authority", &treasury_id.to_bytes()], program_id)
    }

    pub fn verify_fund_authority_address(
        key: &Pubkey,
        treasury_id: &Pubkey,
        program_id: &Pubkey,
    ) -> Result<u8, ProgramError> {
        let (derived_key, seed) = Self::fund_authority_address(treasury_id, program_id);
        if *key != derived_key {
            return Err(TreasuryError::InvalidTreasuryFundAuthorityAddress.into());
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
        let user_treasury = SimpleTreasury {
            mint: Pubkey::new_unique(),
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
            mint: Pubkey::new_unique(),
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
