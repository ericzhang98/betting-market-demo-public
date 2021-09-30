use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum BettingMarketError {
  /// Invalid instruction
  #[error("Invalid Instruction")]
  InvalidInstruction,

  /// Not rent exempt
  #[error("Not rent exempt")]
  NotRentExempt,

  /// Expected amount mismatch
  #[error("Expected amount mismatch")]
  ExpectedAmountMismatch,

  /// Amount overflow
  #[error("Amount overflow")]
  AmountOverflow,

  /// Invalid pda
  #[error("Invalid pda")]
  InvalidPda,
}

impl From<BettingMarketError> for ProgramError {
  fn from(e: BettingMarketError) -> Self {
    ProgramError::Custom(e as u32)
  }
}
