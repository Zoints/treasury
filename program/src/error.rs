use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error, FromPrimitive)]
pub enum TreasuryError {
    /// Invalid instruction
    #[error("Invalid instruction")]
    InvalidInstruction,
    /// Invalid Settings Key
    #[error("Invalid Settings Key")]
    InvalidSettingsKey,
    /// Invalid User Community Key
    #[error("Invalid User Community Key")]
    InvalidUserCommunityKey,
    /// Invalid Zoints Community Key
    #[error("Invalid Zoints Community Key")]
    InvalidZointsCommunityKey,
    /// Name is too short (at least 1 character)
    #[error("Name is too short (at least 1 character)")]
    ZointsCommunityNameTooShort,
    /// Name is too long (max 32 characters)
    #[error("Name is too long (max 32 characters)")]
    ZointsCommunityNameTooLong,
    /// Name contains invalid characters
    #[error("Name contains invalid characters")]
    ZointsCommunityNameInvalidCharacters,
}
impl From<TreasuryError> for ProgramError {
    fn from(e: TreasuryError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for TreasuryError {
    fn type_of() -> &'static str {
        "TreasuryError"
    }
}
