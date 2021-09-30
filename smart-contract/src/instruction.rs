use crate::{error::BettingMarketError::InvalidInstruction, state};
use solana_program::program_error::ProgramError;
use std::convert::TryInto;

pub enum BettingMarketInstruction {
  /// Initializes a betting market
  ///
  ///
  /// Accounts expected:
  ///
  /// 0. `[signer]` The account of the person initiazing the betting market (fee payer)
  /// 1. `[]` The PDA account
  /// 2. `[signer, writable]` The betting market data account to be initialized
  /// 3. `[]` Token program id
  /// 4. `[]` USD token mint already initialized
  /// 5. `[signer, writable]` Yes token mint to be initialized
  /// 6. `[signer, writable]` No token mint to be initialized
  /// 7. `[signer, writable]` USD token account to be initialized
  /// 7. `[]` Judge account to be saved in data
  /// 8. `[]` System program id
  /// 9. `[]` Rent account
  InitBettingMarket {},

  /// Processes a trade
  ///
  ///
  /// Accounts expected:
  ///
  /// 0. `[signer]` The account of the user placing trade
  /// 1. `[]` The PDA account
  /// 2. `[writable]` The betting market data account
  /// 3. `[writable]` USD token mint (why does this need to be writable?)
  /// 4. `[writable]` Yes token mint (why does this need to be writable?)
  /// 5. `[writable]` No token mint (why does this need to be writable?)
  /// 6. `[writable]` The user's USD token account
  /// 7. `[writable]` The user's yes token account
  /// 8. `[writable]` The user's no token account
  /// 9. `[writable]` The betting market's USD token account (owned by PDA)
  /// 10. `[]` Token program id
  OfferTrade {
    is_yes: bool,
    price: u64,
    amount: u64,
  },

  /// Pays out all the payouts for a user
  ///
  ///
  /// Accounts expected:
  ///
  /// 0. `[signer]` The account of the user geting the payout
  /// 1. `[]` The PDA account
  /// 2. `[writable]` The betting market data account
  /// 3. `[writable]` USD token mint (why does this need to be writable?)
  /// 4. `[writable]` Yes token mint (why does this need to be writable?)
  /// 5. `[writable]` No token mint (why does this need to be writable?)
  /// 6. `[writable]` The user's USD token account
  /// 7. `[writable]` The user's yes token account
  /// 8. `[writable]` The user's no token account
  /// 9. `[writable]` The betting market's USD token account (owned by PDA)
  /// 10. `[]` Token program id
  Payout {},

  /// Mint tokens to user for free
  ///
  ///
  /// Accounts expected:
  ///
  /// 0. `[]` The PDA account
  /// 1. `[writable]` Desired token mint (why does this need to be writable?)
  /// 2. `[writable]` The user's desired token account
  /// 3. `[]` Token program id
  FreeMint { amount: u64 },

  /// Set the result of the betting market manually
  ///
  ///
  /// Accounts expected:
  ///
  /// 0. `[writable]` The betting market data account
  JudgeBettingMarketManually { result: u64 },

  /// Set the result of the betting market from oracle
  ///
  ///
  /// Accounts expected:
  ///
  /// 0. `[writable]` The betting market data account
  /// 1. `[]` The pyth price account
  JudgeBettingMarketOracle {},

  /// Set the betting market strike price
  ///
  ///
  /// Accounts expected:
  ///
  /// 0. `[writable]` The betting market data account
  SetStrikePrice { strike_price: u64 },
}

impl BettingMarketInstruction {
  /// Unpacks a byte buffer into a [BettingMarketInstruction](enum.BettingMarketInstruction.html).
  pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
    let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
    let rest_ptr = rest.as_ptr();

    Ok(match tag {
      2 => Self::InitBettingMarket {},
      3 => Self::OfferTrade {
        is_yes: state::get_u8_at_ptr(rest_ptr) == 1,
        price: state::get_u64_at_ptr_offset(rest_ptr, 1),
        amount: state::get_u64_at_ptr_offset(rest_ptr, 9),
      },
      4 => Self::Payout {},
      5 => Self::FreeMint {
        amount: Self::unpack_amount(rest)?,
      },
      6 => Self::JudgeBettingMarketManually {
        result: Self::unpack_amount(rest)?,
      },
      7 => Self::JudgeBettingMarketOracle {},
      8 => Self::SetStrikePrice {
        strike_price: Self::unpack_amount(rest)?,
      },
      _ => return Err(InvalidInstruction.into()),
    })
  }

  fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
    let amount = input
      .get(..8)
      .and_then(|slice| slice.try_into().ok())
      .map(u64::from_le_bytes)
      .ok_or(InvalidInstruction)?;
    Ok(amount)
  }
}
