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
    /// Invalid User Treasury Key
    #[error("Invalid User Treasury Key")]
    InvalidUserTreasuryKey,
    /// User Treasury Already Exists
    #[error("User Treasury Already Exists")]
    UserTreasuryExists,
    /// Invalid Zoints Treasury Key
    #[error("Invalid Zoints Treasury Key")]
    InvalidZointsTreasuryKey,
    /// Zoints Treasury Already Exists
    #[error("Zoints Treasury Already Exists")]
    ZointsTreasuryExists,
    /// Name is too short (at least 1 character)
    #[error("Name is too short (at least 1 character)")]
    ZointsTreasuryNameTooShort,
    /// Name is too long (max 32 characters)
    #[error("Name is too long (max 32 characters)")]
    ZointsTreasuryNameTooLong,
    /// Name contains invalid characters
    #[error("Name contains invalid characters")]
    ZointsTreasuryNameInvalidCharacters,
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

    /// Invalid Fee Recipient
    #[error("Invalid Fee Recipient")]
    InvalidFeeRecipient,

    /// Invalid Price Authority
    #[error("Invalid Price Authority")]
    InvalidPriceAuthority,
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
