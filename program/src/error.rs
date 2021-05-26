use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error, FromPrimitive)]
pub enum TreasuryError {
    /// Invalid instruction
    #[error("Invalid instruction")]
    InvalidInstruction,
    /// Program has already been initialized
    #[error("Program has already been initialized")]
    AlreadyInitialized,
    /// Program has not been initialized
    #[error("Program has not been initialized")]
    NotInitialized,
    /// Invalid Settings Key
    #[error("Invalid Settings Key")]
    InvalidSettingsKey,
    /// The authority did not sign the transaction
    #[error("The authority did not sign the transaction")]
    MissingAuthoritySignature,
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
    /// The token is not a valid SPL Token Mint
    #[error("The token is not a valid SPL Token Mint")]
    TokenNotSPLToken,
    /// Missing Creator's Signature
    #[error("Missing Creator's Signature")]
    MissingCreatorSignature,
    /// Associated account is invalid
    #[error("Associated account is invalid")]
    AssociatedAccountInvalid,
    /// Associated account is for the wrong Mint
    #[error("Associated account is for the wrong Mint")]
    AssociatedAccountWrongMint,
    /// Mint is invalid
    #[error("Mint is invalid")]
    MintInvalid,
    /// Mint is for the wrong token
    #[error("Mint is for the wrong token")]
    MintWrongToken,

    /// Not enough ZEE to cover the cost of creation
    #[error("Not enough ZEE to cover the cost of creation")]
    NotEnoughZEE,
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
