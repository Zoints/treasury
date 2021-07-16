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

    /// Invalid Treasury Owner
    #[error("Invalid Treasury Owner")]
    InvalidTreasuryOwner,

    /// Invalid Treasury Fund Authority Address
    #[error("Invalid Treasury Fund Authority Address")]
    InvalidTreasuryFundAuthorityAddress,

    /// Invalid Treasury Fund Address
    #[error("Invalid Treasury Fund Address")]
    InvalidTreasuryFundAddress,

    /// Invalid Treasury Fund Account
    #[error("Invalid Treasury Fund Account")]
    InvalidTreasuryFundAccount,

    /// Treasury Already Exists
    #[error("Treasury Already Exists")]
    TreasuryAlreadyExists,

    /// The token is not a valid SPL Token Mint
    #[error("The token is not a valid SPL Token Mint")]
    TokenNotSPLToken,

    /// Mint is invalid
    #[error("Mint is invalid")]
    MintInvalid,

    /// Mint is for the wrong token
    #[error("Mint is for the wrong token")]
    MintWrongToken,

    /// Invalid Vestment Percentage (must be between 1 and 10,000)
    #[error("Invalid Vestment Percentage (must be between 1 and 10,000)")]
    InvalidVestmentPercentage,

    /// Invalid Vestment Period (must be > 0)
    #[error("Invalid Vestment Invalid Vestment Period (must be > 0)")]
    InvalidVestmentPeriod,

    /// Invalid Vestment Amount (must be > 0)
    #[error("Invalid Vestment Amount (must be > 0)")]
    InvalidVestmentAmount,

    /// Invalid Recipient
    #[error("Invalid Recipient")]
    InvalidRecipient,

    /// Invalid Recipient Account
    #[error("Invalid Recipient Account")]
    InvalidRecipientAccount,
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
