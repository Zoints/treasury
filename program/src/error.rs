use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError, msg, program_error::PrintProgramError, program_error::ProgramError,
};
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

    /// Invalid Treasury Address
    #[error("Invalid Treasury Address")]
    InvalidTreasuryAddress,

    /// Invalid Treasury Fund Address
    #[error("Invalid Treasury Fund Address")]
    InvalidTreasuryFundAddress,

    /// User Treasury Already Exists
    #[error("User Treasury Already Exists")]
    UserTreasuryExists,

    /// The token is not a valid SPL Token Mint
    #[error("The token is not a valid SPL Token Mint")]
    TokenNotSPLToken,
    /// Missing Creator's Signature
    #[error("Missing Creator's Signature")]
    MissingCreatorSignature,
    /// Associated account is invalid
    #[error("Associated account is invalid")]
    AssociatedAccountInvalid,
    /// Treasury associated account is invalid
    #[error("Treasury associated account is invalid")]
    TreasuryAssociatedAccountInvalid,
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

impl PrintProgramError for TreasuryError {
    fn print<E>(&self) {
        msg!("TREASURY-ERROR: {}", &self.to_string());
    }
}
