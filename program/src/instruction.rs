use crate::error::TreasuryError;
use arrayref::array_ref;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::pubkey::PUBKEY_BYTES;
use std::convert::TryInto;
use std::mem::size_of;

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum TreasuryInstruction {
    Initialize { fee_user: u64, fee_zoints: u64 },
    CreateUserTreasury,
    CreateZointsTreasury { name: Vec<u8> },
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

                let (fee_zoints, rest) = rest.split_at(8);
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
