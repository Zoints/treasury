use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error, FromPrimitive)]
pub enum TreasuryError {
    /// Invalid instruction
    #[error("Invalid instruction")]
    InvalidInstruction,
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
