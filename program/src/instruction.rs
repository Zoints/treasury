use crate::error::TreasuryError;
use arrayref::array_ref;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use std::convert::TryInto;
use std::mem::size_of;

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum TreasuryInstruction {
    /// Initialize the Treasury Program
    ///
    /// This only needs to be done once to set the initial parameters
    ///
    /// Accounts expected by this instruction:
    ///   0. `[signer]` The account funding the instruction
    ///   1. `[]` The ZEE Token mint
    ///   2. `[signer]` The authority that sets fees
    ///   3. `[]` The address that receives fees (must be valid ZEE associated account)
    ///   4. `[writable]` The global settings program account
    ///   5. `[]` Rent sysvar
    Initialize { fee_user: u64, fee_zoints: u64 },
    /// Create User Treasury
    ///
    /// Initializes a treasury for a specific user. SOL fees are paid by the funder
    /// and ZEE fees are paid by the creator
    ///
    /// Accounts expected by this instruction:
    ///   0. `[signer]` The account funding the instruction
    ///   1. `[signer]` The creator
    ///   2. `[writable]` The creator's ZEE associated token account
    ///   3. `[]` The ZEE token mint
    ///   4. `[]` The global settings program account
    ///   5. `[writable]` The fee recipient address
    ///   6. `[]` Rent sysvar
    ///   7. `[]` The SPL Token program
    ///   8. `[]` System Program
    CreateUserTreasury,
    /// Create Zoints Treasury
    ///
    /// Initializes a treasury for a zoints community. SOL fees are paid by the funder
    /// and ZEE fees are paid by the creator.
    ///
    /// Names are unique, first come first serve. Max 32 bytes.
    ///
    /// Accounts expected by this instruction:
    ///   0. `[signer]` The account funding the instruction
    ///   1. `[signer]` The creator
    ///   2. `[writable]` The creator's ZEE associated token account
    ///   3. `[]` The ZEE token mint
    ///   4. `[]` The global settings program account
    ///   5. `[writable]` The fee recipient address
    ///   6. `[]` Rent sysvar
    ///   7. `[]` The SPL Token program
    ///   8. `[]` System Program
    CreateZointsTreasury { name: Vec<u8> },
    /// Update Fees and Fee Recipient
    ///
    /// Accounts expected by this instruction:
    ///   0. `[signer]` The account funding the instruction
    ///   1. `[signer]` The authority that sets fees
    ///   2. `[]` The address that receives fees (must be valid ZEE associated account)
    ///   3. `[writable]` The global settings program account
    UpdateFees { fee_user: u64, fee_zoints: u64 },
}

impl TreasuryInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        use TreasuryError::InvalidInstruction;

        let (&ins, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match ins {
            0 => {
                let (fee_user, rest) = rest.split_at(8);
                let fee_user = fee_user
                    .try_into()
                    .ok()
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;

                let (fee_zoints, _rest) = rest.split_at(8);
                let fee_zoints = fee_zoints
                    .try_into()
                    .ok()
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;

                TreasuryInstruction::Initialize {
                    fee_user,
                    fee_zoints,
                }
            }
            1 => TreasuryInstruction::CreateUserTreasury,
            2 => {
                let (name, _rest) = unpack_vec(rest)?;
                TreasuryInstruction::CreateZointsTreasury { name }
            }
            3 => {
                let (fee_user, rest) = rest.split_at(8);
                let fee_user = fee_user
                    .try_into()
                    .ok()
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;

                let (fee_zoints, _rest) = rest.split_at(8);
                let fee_zoints = fee_zoints
                    .try_into()
                    .ok()
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;

                TreasuryInstruction::UpdateFees {
                    fee_user,
                    fee_zoints,
                }
            }
            _ => return Err(InvalidInstruction.into()),
        })
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            &TreasuryInstruction::Initialize {
                fee_user,
                fee_zoints,
            } => {
                buf.push(0);
                buf.extend_from_slice(&fee_user.to_le_bytes());
                buf.extend_from_slice(&fee_zoints.to_le_bytes());
            }
            TreasuryInstruction::CreateUserTreasury => buf.push(1),
            TreasuryInstruction::CreateZointsTreasury { name } => {
                buf.push(2);
                pack_vec(&name, &mut buf);
            }
            TreasuryInstruction::UpdateFees {
                fee_user,
                fee_zoints,
            } => {
                buf.push(3);
                buf.extend_from_slice(&fee_user.to_le_bytes());
                buf.extend_from_slice(&fee_zoints.to_le_bytes());
            }
        }
        buf
    }
}

pub fn unpack_vec(input: &[u8]) -> Result<(Vec<u8>, &[u8]), ProgramError> {
    if input.len() < 2 {
        msg!("no len data for vector");
        return Err(ProgramError::InvalidInstructionData);
    }
    let (len, rest) = input.split_at(2);
    let len = array_ref![len, 0, 2];
    let len = u16::from_le_bytes(*len) as usize;

    if rest.len() < len {
        msg!(
            "data too short for len. len = {}, actual = {}",
            len,
            rest.len()
        );
        return Err(ProgramError::InvalidInstructionData);
    }
    let (data, rest) = rest.split_at(len);

    Ok((Vec::from(data), rest))
}

pub fn pack_vec(value: &Vec<u8>, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&(value.len() as u16).to_le_bytes());
    buf.extend_from_slice(&value);
}
