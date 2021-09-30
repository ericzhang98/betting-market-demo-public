use crate::{
  error::BettingMarketError,
  instruction::BettingMarketInstruction,
  state::{
    self, get_payout_at_index, get_u64_at_ptr_offset, get_u8_at_ptr_offset, payout_exists_at_index,
    set_payout_at_index, set_pubkey_10_at_ptr_offset, set_u64_10_at_ptr_offset,
    set_u64_at_ptr_offset, set_u8_at_ptr_offset, BettingMarket, NO_BUY_AMOUNT_START_OFFSET,
    NULL_PUBKEY, PAYOUT_AMOUNTS_FOR_PRICE_START_OFFSET, PAYOUT_IN_USD_FOR_PRICE_START_OFFSET,
    PUBKEY_10_ISIZE, U64_10_ISIZE, U64_ISIZE, USER_ACCOUNTS_FOR_PRICE_START_OFFSET,
    YES_BUY_AMOUNT_START_OFFSET,
  },
};
use solana_program::{
  account_info::{next_account_info, AccountInfo},
  entrypoint::ProgramResult,
  log::sol_log_compute_units,
  msg,
  program::{invoke, invoke_signed},
  program_error::ProgramError,
  program_pack::Pack,
  pubkey::Pubkey,
  system_instruction,
  sysvar::{rent::Rent, Sysvar},
};
use spl_token::{state::Account as TokenAccount, state::Mint as TokenMintAccount};
use std::convert::TryInto;
use std::str::FromStr;

pub struct Processor;
impl Processor {
  pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
  ) -> ProgramResult {
    let instruction = BettingMarketInstruction::unpack(instruction_data)?;

    match instruction {
      BettingMarketInstruction::InitBettingMarket {} => {
        msg!("Instruction: InitBettingMarket");
        Self::process_init_betting_market(accounts, program_id)
      }
      BettingMarketInstruction::OfferTrade {
        is_yes,
        price,
        amount,
      } => {
        msg!("Instruction: OfferTrade");
        msg!("is_yes: {}", is_yes);
        msg!("price: {}", price);
        msg!("amount: {}", amount);
        Self::process_offer_trade(accounts, is_yes, price, amount, program_id)
      }
      BettingMarketInstruction::Payout {} => {
        msg!("Instruction: Payout");
        Self::process_payout(accounts, program_id)
      }
      BettingMarketInstruction::FreeMint { amount } => {
        msg!("Instruction: FreeMint");
        msg!("amount: {}", amount);
        Self::process_free_mint(accounts, amount, program_id)
      }
      BettingMarketInstruction::JudgeBettingMarketManually { result } => {
        msg!("Instruction: JudgeBettingMarketManually");
        msg!("result: {}", result);
        Self::process_judge_betting_market_manually(accounts, result)
      }
      BettingMarketInstruction::JudgeBettingMarketOracle {} => {
        msg!("Instruction: JudgeBettingMarketOracle");
        Self::process_judge_betting_market_oracle(accounts)
      }
      BettingMarketInstruction::SetStrikePrice { strike_price } => {
        msg!("Instruction: SetStrikePrice");
        msg!("strike price: {}", strike_price);
        Self::process_set_strike_price(accounts, strike_price)
      }
    }
  }

  // initialize betting market data account, yes/no token mints, usd token account
  fn process_init_betting_market(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let initializer_account_info = next_account_info(account_info_iter)?;
    let pda_account_info = next_account_info(account_info_iter)?;
    let betting_market_data_account_info = next_account_info(account_info_iter)?;
    let token_program_id_account_info = next_account_info(account_info_iter)?;
    let usd_token_mint_account_info = next_account_info(account_info_iter)?;
    let yes_token_mint_account_info = next_account_info(account_info_iter)?;
    let no_token_mint_account_info = next_account_info(account_info_iter)?;
    let usd_token_account_info = next_account_info(account_info_iter)?;
    let judge_account_info = next_account_info(account_info_iter)?;
    let system_program_account_info = next_account_info(account_info_iter)?;
    let rent_account_info = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(rent_account_info)?;
    let (pda, _bump_seed) = Pubkey::find_program_address(&[b"betting"], program_id);

    // verify pda is indeed pda, token program id is correct, token mint accounts are owned by token program
    // initializer is signer, anything already initialized?
    if *pda_account_info.key != pda {
      return Err(BettingMarketError::InvalidPda.into());
    }
    if *token_program_id_account_info.key != spl_token::id() {
      return Err(ProgramError::IncorrectProgramId);
    }
    if !initializer_account_info.is_signer {
      return Err(ProgramError::MissingRequiredSignature);
    }

    // create account + initialize with pda as mint authority / owner instead of initializer so no need to set authority
    // accounts to be initialized are thus required to be writable

    // create token mint accounts and initialize mint with pda as mint authority
    msg!("Creating token mint accounts and initializing mint with pda as mint authority");
    invoke(
      &system_instruction::create_account(
        initializer_account_info.key,
        yes_token_mint_account_info.key,
        rent.minimum_balance(TokenMintAccount::LEN),
        TokenMintAccount::LEN as u64,
        token_program_id_account_info.key,
      ),
      &[
        initializer_account_info.clone(),
        yes_token_mint_account_info.clone(),
        system_program_account_info.clone(),
      ],
    )?;
    invoke(
      &spl_token::instruction::initialize_mint(
        token_program_id_account_info.key,
        yes_token_mint_account_info.key,
        pda_account_info.key,
        None,
        2,
      )?,
      &[
        yes_token_mint_account_info.clone(),
        rent_account_info.clone(),
        token_program_id_account_info.clone(),
      ],
    )?;
    invoke(
      &system_instruction::create_account(
        initializer_account_info.key,
        no_token_mint_account_info.key,
        rent.minimum_balance(TokenMintAccount::LEN),
        TokenMintAccount::LEN as u64,
        token_program_id_account_info.key,
      ),
      &[
        initializer_account_info.clone(),
        no_token_mint_account_info.clone(),
        system_program_account_info.clone(),
      ],
    )?;
    invoke(
      &spl_token::instruction::initialize_mint(
        token_program_id_account_info.key,
        no_token_mint_account_info.key,
        pda_account_info.key,
        None,
        2,
      )?,
      &[
        no_token_mint_account_info.clone(),
        rent_account_info.clone(),
        token_program_id_account_info.clone(),
      ],
    )?;

    // create usd token account and initialize token account with pda as owner
    msg!("Creating usd token account and initializing token account with pda as owner");
    invoke(
      &system_instruction::create_account(
        initializer_account_info.key,
        usd_token_account_info.key,
        rent.minimum_balance(TokenAccount::LEN),
        TokenAccount::LEN as u64,
        token_program_id_account_info.key,
      ),
      &[
        initializer_account_info.clone(),
        usd_token_account_info.clone(),
        system_program_account_info.clone(),
      ],
    )?;
    invoke(
      &spl_token::instruction::initialize_account(
        token_program_id_account_info.key,
        usd_token_account_info.key,
        usd_token_mint_account_info.key,
        pda_account_info.key,
      )?,
      &[
        usd_token_account_info.clone(),
        usd_token_mint_account_info.clone(),
        pda_account_info.clone(),
        rent_account_info.clone(),
        token_program_id_account_info.clone(),
      ],
    )?;

    // verify betting market data account owned by program and initialize its data
    msg!("Verifying betting market data account owned by program and initializing its data");
    if *betting_market_data_account_info.owner != *program_id {
      msg!("wtf!");
      msg!(
        "{} {}",
        *betting_market_data_account_info.owner,
        *program_id
      );
      // return Err(ProgramError::IllegalOwner);
    }
    let mut_ptr = betting_market_data_account_info
      .data
      .borrow_mut()
      .as_mut_ptr();
    unsafe {
      let slice = std::slice::from_raw_parts_mut(mut_ptr, BettingMarket::LEN);
      let mut betting_market_data = BettingMarket::unpack_unchecked(slice)?;
      // if betting_market_data.is_initialized() {
      //   return Err(ProgramError::AccountAlreadyInitialized);
      // }
      msg!("{}", betting_market_data.is_initialized);
      betting_market_data.is_initialized = true;
      betting_market_data.result = 0;
      betting_market_data.yes_token_mint = *yes_token_mint_account_info.key;
      betting_market_data.no_token_mint = *no_token_mint_account_info.key;
      betting_market_data.usd_token_account = *usd_token_account_info.key;
      betting_market_data.strike_price = 0;
      betting_market_data.judge = *judge_account_info.key;
      BettingMarket::pack(betting_market_data, slice)?;
    }

    // hardcoded YES and NO token mints
    // state::set_pubkey_at_ptr_offset(
    //   mut_ptr,
    //   2,
    //   Pubkey::from_str(REPLACE_ME).unwrap(),
    // );
    // state::set_pubkey_at_ptr_offset(
    //   mut_ptr,
    //   34,
    //   Pubkey::from_str(REPLACE_ME).unwrap(),
    // );

    sol_log_compute_units();

    Ok(())
  }

  // process an offer trade instruction by filling as much of the amount as possible under the limit price
  // and creating the remaining amount as a resting limit order at the specified limit price
  // every trade is a buy (selling yes for 30 == buying no for 70)
  // with the direction specified by is_yes
  fn process_offer_trade(
    accounts: &[AccountInfo],
    is_yes: bool,
    price: u64,
    amount: u64,
    program_id: &Pubkey,
  ) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let user_account = next_account_info(account_info_iter)?;
    let pda_account = next_account_info(account_info_iter)?;
    let betting_market_data_account = next_account_info(account_info_iter)?;
    let usd_token_mint = next_account_info(account_info_iter)?;
    let yes_token_mint = next_account_info(account_info_iter)?;
    let no_token_mint = next_account_info(account_info_iter)?;
    let user_usd_token_account = next_account_info(account_info_iter)?;
    let user_yes_token_account = next_account_info(account_info_iter)?;
    let user_no_token_account = next_account_info(account_info_iter)?;
    let pda_usd_token_account = next_account_info(account_info_iter)?;
    let token_program_id = next_account_info(account_info_iter)?;

    // TODO: checks
    // valid token accounts, pda account is indeed pda, betting market data account owned by program
    // token mints match up correctly, usd pda usd token account owned by pda
    let (pda, _bump_seed) = Pubkey::find_program_address(&[b"betting"], program_id);
    if *pda_account.key != pda {
      return Err(BettingMarketError::InvalidPda.into());
    }

    // unpack token account data
    let user_yes_token_account_data = TokenAccount::unpack(&user_yes_token_account.data.borrow())?;
    let user_no_token_account_data = TokenAccount::unpack(&user_no_token_account.data.borrow())?;
    let user_usd_token_account_data = TokenAccount::unpack(&user_usd_token_account.data.borrow())?;

    // when collecting payment, attempt inverse selling first before buying with usd
    let (
      forward_buy_mint,
      forward_buy_account,
      forward_buy_account_data,
      inverse_sell_mint,
      inverse_sell_account,
      inverse_sell_account_data,
    ) = if is_yes {
      (
        yes_token_mint,
        user_yes_token_account,
        user_yes_token_account_data,
        no_token_mint,
        user_no_token_account,
        user_no_token_account_data,
      )
    } else {
      (
        no_token_mint,
        user_no_token_account,
        user_no_token_account_data,
        yes_token_mint,
        user_yes_token_account,
        user_yes_token_account_data,
      )
    };

    msg!(
      "usd mint: {}, account: {}, balance: {}",
      usd_token_mint.key,
      user_usd_token_account.key,
      user_usd_token_account_data.amount,
    );
    msg!(
      "forward buy mint: {}, account: {}, balance: {}",
      forward_buy_mint.key,
      forward_buy_account.key,
      forward_buy_account_data.amount,
    );
    msg!(
      "inverse sell mint: {}, account: {}, balance: {}",
      inverse_sell_mint.key,
      inverse_sell_account.key,
      inverse_sell_account_data.amount,
    );

    // burn inverse sell amount, which is min(inverse sell account balance, amount)
    let inverse_collateralized_amount = std::cmp::min(inverse_sell_account_data.amount, amount);
    msg!("inverse sell amount: {}", inverse_collateralized_amount);
    invoke(
      &spl_token::instruction::burn(
        token_program_id.key,
        inverse_sell_account.key,
        &inverse_sell_account_data.mint,
        user_account.key,
        &[],
        inverse_collateralized_amount,
      )?,
      &[
        inverse_sell_account.clone(),
        inverse_sell_mint.clone(),
        user_account.clone(),
        token_program_id.clone(),
      ],
    )?;
    msg!("burned {} inverse tokens", inverse_collateralized_amount);

    // transfer remaining amount in USD to betting market's USD token account
    let usd_collateralized_amount = amount - inverse_collateralized_amount;
    let usd_amount = price * usd_collateralized_amount; // can overflow
    msg!(
      "usd collateralized amount: {}, price: {}, usd amount: {}",
      usd_collateralized_amount,
      price,
      usd_amount
    );
    invoke(
      &spl_token::instruction::transfer(
        token_program_id.key,
        user_usd_token_account.key,
        pda_usd_token_account.key,
        user_account.key,
        &[],
        usd_amount,
      )?,
      &[
        user_usd_token_account.clone(),
        pda_usd_token_account.clone(),
        user_account.clone(),
        token_program_id.clone(),
      ],
    )?;
    msg!("transferred {} usd", usd_amount);

    // match trades and adjust order book
    // place 2 separate orders -- 1 for the inverse collateralized amount and 1 for the usd collateralized amount
    let data_ptr = betting_market_data_account.data.borrow_mut().as_mut_ptr();
    msg!(
      "first trade collateralized by inverse {} token burn and payout in usd",
      if is_yes { "no" } else { "yes" }
    );
    Self::match_and_place_limit_order(
      is_yes,
      inverse_collateralized_amount,
      price,
      true,
      user_account,
      data_ptr,
      *yes_token_mint.key,
      *no_token_mint.key,
      *usd_token_mint.key,
    );
    msg!(
      "second trade collateralized by usd transfer and payout in {} token",
      if is_yes { "yes" } else { "no" }
    );
    Self::match_and_place_limit_order(
      is_yes,
      usd_collateralized_amount,
      price,
      false,
      user_account,
      data_ptr,
      *yes_token_mint.key,
      *no_token_mint.key,
      *usd_token_mint.key,
    );

    Ok(())
  }

  // match as much of the limit order as possible by crossing over and then
  // place the remaining unmatched amount as a resting limit order on the order book
  // Access violation in unknown section at address 0xfffffffffffff017 of size 8 by instruction #7094
  // wtf the program crashes if I don't put this
  #[inline(always)]
  fn match_and_place_limit_order(
    is_yes: bool,
    order_size: u64,
    limit_price: u64,
    order_payout_in_usd: bool,
    user_account: &AccountInfo,
    data_ptr: *mut u8,
    yes_token_mint: Pubkey,
    no_token_mint: Pubkey,
    usd_token_mint: Pubkey,
  ) {
    let mut matched_amount = 0;
    let mut unmatched_amount = order_size;
    let (forward_token_mint, inverse_token_mint) = if is_yes {
      (yes_token_mint, no_token_mint)
    } else {
      (no_token_mint, yes_token_mint)
    };

    // iterate thru inverse buy orders from 100 to (100-limit price) inclusive
    // while unmatched_amount > 0, perform trade
    // trade will fully exhaust either one or both sides
    // the inverse side will have exactly 1 settled result per order filled (same price can have multiple orders)
    // the forward side will have either 1 or 2 settled results per order filled
    // if it's a usd payout order, it'll be 1 payout at the forward buy price
    // if it's a forward buy mint order, it'll be 1 payout in forward buy tokens and 1 payout for the price differential
    // update the order book and append the settled results to payout data
    for inverse_buy_price in ((100 - limit_price)..101).rev() {
      let inverse_buy_price_isize = inverse_buy_price as isize;
      let inverse_buy_price_amount_offset = if is_yes {
        NO_BUY_AMOUNT_START_OFFSET
      } else {
        YES_BUY_AMOUNT_START_OFFSET
      } + U64_ISIZE * inverse_buy_price_isize;
      let amount_at_inverse_buy_price =
        state::get_u64_at_ptr_offset(data_ptr, inverse_buy_price_amount_offset);
      let matched_at_price = std::cmp::min(unmatched_amount, amount_at_inverse_buy_price);
      matched_amount += matched_at_price;
      unmatched_amount -= matched_at_price;

      if matched_at_price > 0 {
        // update buy amounts for price
        state::set_u64_at_ptr_offset(
          data_ptr,
          inverse_buy_price_amount_offset,
          amount_at_inverse_buy_price - matched_at_price,
        );
        msg!(
          "matched {} inverse buys at price {}, updating amount at {} buy price from {} to {}",
          matched_at_price,
          inverse_buy_price,
          if is_yes { "no" } else { "yes" },
          amount_at_inverse_buy_price,
          amount_at_inverse_buy_price - matched_at_price,
        );

        // match orders for the inverse token on the orderbook at inverse buy price and create corresponding payouts
        Self::match_orders_at_price_fifo(
          data_ptr,
          matched_at_price,
          inverse_buy_price,
          inverse_token_mint,
          usd_token_mint,
        );

        // create payout for user at forward buy price
        let forward_buy_price = 100 - inverse_buy_price; // cross over at better than limit price
        let forward_buy_price_differential = limit_price - forward_buy_price; // geq 0
        if order_payout_in_usd {
          Self::add_payout(
            data_ptr,
            *user_account.key,
            usd_token_mint,
            matched_at_price * forward_buy_price,
          );
        } else {
          Self::add_payout(
            data_ptr,
            *user_account.key,
            forward_token_mint,
            matched_at_price,
          );
          // usd price differential paid back
          Self::add_payout(
            data_ptr,
            *user_account.key,
            usd_token_mint,
            matched_at_price * forward_buy_price_differential,
          )
        }
      }
    }

    // add remaining unmatched amount as order to order book
    if unmatched_amount > 0 {
      Self::add_order_to_orderbook(
        user_account,
        data_ptr,
        is_yes,
        limit_price,
        unmatched_amount,
        order_payout_in_usd,
      );
    }

    msg!(
      "matched amount: {}, unmatched amount: {}",
      matched_amount,
      unmatched_amount
    );
  }

  // match orders at a fixed price from the orderbook in fifo fashion for the order size and create corresponding payouts
  fn match_orders_at_price_fifo(
    data_ptr: *mut u8,
    order_size: u64,
    order_price: u64,
    order_token_mint: Pubkey,
    usd_token_mint: Pubkey,
  ) {
    let order_price_isize = order_price as isize;
    let user_accounts_for_price_offset =
      USER_ACCOUNTS_FOR_PRICE_START_OFFSET + PUBKEY_10_ISIZE * order_price_isize;
    let payout_in_usd_for_price_offset =
      PAYOUT_IN_USD_FOR_PRICE_START_OFFSET + U64_10_ISIZE * order_price_isize;
    let payout_amounts_for_price_offset =
      PAYOUT_AMOUNTS_FOR_PRICE_START_OFFSET + U64_10_ISIZE * order_price_isize;
    let mut user_accounts_for_price =
      state::get_pubkey_10_at_ptr_offset(data_ptr, user_accounts_for_price_offset);
    let mut payout_in_usd_for_price =
      state::get_u64_10_at_ptr_offset(data_ptr, payout_in_usd_for_price_offset);
    let mut payout_amounts_for_price =
      state::get_u64_10_at_ptr_offset(data_ptr, payout_amounts_for_price_offset);
    let mut unmatched_order_buys = order_size;
    while unmatched_order_buys > 0 {
      let matched_for_order = std::cmp::min(payout_amounts_for_price[0], unmatched_order_buys);
      Self::add_payout(
        data_ptr,
        user_accounts_for_price[0],
        if payout_in_usd_for_price[0] == 2 {
          usd_token_mint
        } else {
          order_token_mint
        },
        if payout_in_usd_for_price[0] == 2 {
          matched_for_order * (100 - order_price) // original order was selling inverse, so usd amount is inverse
        } else {
          matched_for_order
        },
      );
      unmatched_order_buys -= matched_for_order;
      payout_amounts_for_price[0] -= matched_for_order;
      // pop from front if order fully matched
      if payout_amounts_for_price[0] == 0 {
        for i in 0..9 {
          user_accounts_for_price[i] = user_accounts_for_price[i + 1];
          payout_in_usd_for_price[i] = payout_in_usd_for_price[i + 1];
          payout_amounts_for_price[i] = payout_amounts_for_price[i + 1];
        }
        user_accounts_for_price[9] = NULL_PUBKEY;
        payout_in_usd_for_price[9] = 0;
        payout_amounts_for_price[9] = 0;
      }
    }
    set_pubkey_10_at_ptr_offset(
      data_ptr,
      user_accounts_for_price_offset,
      user_accounts_for_price,
    );
    set_u64_10_at_ptr_offset(
      data_ptr,
      payout_in_usd_for_price_offset,
      payout_in_usd_for_price,
    );
    set_u64_10_at_ptr_offset(
      data_ptr,
      payout_amounts_for_price_offset,
      payout_amounts_for_price,
    );
  }

  // add order info to orderbook and update the buy amount at the order price
  fn add_order_to_orderbook(
    user_account: &AccountInfo,
    data_ptr: *mut u8,
    is_yes: bool,
    price: u64,
    order_size: u64,
    should_payout_in_usd: bool,
  ) {
    if order_size == 0 {
      return;
    }
    msg!(
      "orderbook: adding {} {} tokens at price {} with payout in {}{}",
      order_size,
      if is_yes { "yes" } else { "no" },
      price,
      if should_payout_in_usd {
        "usd"
      } else {
        if is_yes {
          "yes tokens"
        } else {
          "no tokens"
        }
      },
      ""
    );
    let price_isize = price as isize;
    let buy_amount_offset = if is_yes {
      YES_BUY_AMOUNT_START_OFFSET
    } else {
      NO_BUY_AMOUNT_START_OFFSET
    } + U64_ISIZE * price_isize;
    let current_buy_amount = state::get_u64_at_ptr_offset(data_ptr, buy_amount_offset);
    let updated_buy_amount = current_buy_amount + order_size;
    state::set_u64_at_ptr_offset(data_ptr, buy_amount_offset, updated_buy_amount);
    // msg!(
    //   "orderbook updated: buy amount from {} to {} for price {}",
    //   current_buy_amount,
    //   updated_buy_amount,
    //   price
    // );
    // append to user_account_for_price
    let user_accounts_for_price_offset =
      USER_ACCOUNTS_FOR_PRICE_START_OFFSET + PUBKEY_10_ISIZE * price_isize;
    let mut user_accounts_for_price =
      state::get_pubkey_10_at_ptr_offset(data_ptr, user_accounts_for_price_offset);
    // TODO: replace with https://programming-idioms.org/idiom/223/for-else-loop/3852/rust
    for i in 0..10 {
      // msg!(
      //   "user_accounts_for_price[{}] = {}",
      //   i,
      //   user_accounts_for_price[i]
      // );
      if user_accounts_for_price[i] == NULL_PUBKEY {
        user_accounts_for_price[i] = *user_account.key;
        // msg!(
        //   "orderbook updated: user_accounts_for_price[{}] = {}",
        //   i,
        //   user_accounts_for_price[i]
        // );
        break;
      }
    }
    set_pubkey_10_at_ptr_offset(
      data_ptr,
      user_accounts_for_price_offset,
      user_accounts_for_price,
    );
    // append to payout_in_usd_for_price bool {1, 2}
    let payout_in_usd_for_price_offset =
      PAYOUT_IN_USD_FOR_PRICE_START_OFFSET + U64_10_ISIZE * price_isize;
    let mut payout_in_usd_for_price =
      state::get_u64_10_at_ptr_offset(data_ptr, payout_in_usd_for_price_offset);
    for i in 0..10 {
      // msg!("payout_in_usd_for_price[{}] = {}", i, payout_in_usd_for_price[i]);
      if payout_in_usd_for_price[i] == 0 {
        payout_in_usd_for_price[i] = if should_payout_in_usd { 2 } else { 1 };
        // msg!(
        //   "order updated: payout_in_usd_for_price[{}] = {}",
        //   i,
        //   payout_in_usd_for_price[i]
        // );
        break;
      }
    }
    set_u64_10_at_ptr_offset(
      data_ptr,
      payout_in_usd_for_price_offset,
      payout_in_usd_for_price,
    );
    // append to payout_amounts_for_price
    let payout_amounts_for_price_offset =
      PAYOUT_AMOUNTS_FOR_PRICE_START_OFFSET + U64_10_ISIZE * price_isize;
    let mut payout_amounts_for_price =
      state::get_u64_10_at_ptr_offset(data_ptr, payout_amounts_for_price_offset);
    for i in 0..10 {
      // msg!(
      //   "payout_amounts_for_price[{}] = {}",
      //   i,
      //   payout_amounts_for_price[i]
      // );
      if payout_amounts_for_price[i] == 0 {
        payout_amounts_for_price[i] = order_size;
        // msg!(
        //   "orderbook updated: payout_amounts_for_price[{}] = {}",
        //   i,
        //   payout_amounts_for_price[i]
        // );
        break;
      }
    }
    set_u64_10_at_ptr_offset(
      data_ptr,
      payout_amounts_for_price_offset,
      payout_amounts_for_price,
    );
  }

  // add payout info at first free index
  fn add_payout(
    data_ptr: *mut u8,
    payout_user_account: Pubkey,
    payout_mint: Pubkey,
    payout_amount: u64,
  ) {
    if payout_amount == 0 {
      return;
    }
    for i in 0..100 {
      if !payout_exists_at_index(data_ptr, i) {
        set_payout_at_index(data_ptr, i, payout_user_account, payout_mint, payout_amount);
        msg!("added payout info to index {}", i);
        // msg!(
        //   "payout user account: {}, payout mint: {}, payout amount: {}",
        //   payout_user_account,
        //   payout_mint,
        //   payout_amount
        // );
        break;
      }
    }
  }

  // iterate thru payout data and pay out all the payouts for the user
  fn process_payout(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let user_account = next_account_info(account_info_iter)?;
    let pda_account = next_account_info(account_info_iter)?;
    let betting_market_data_account = next_account_info(account_info_iter)?;
    let usd_token_mint = next_account_info(account_info_iter)?;
    let yes_token_mint = next_account_info(account_info_iter)?;
    let no_token_mint = next_account_info(account_info_iter)?;
    let user_usd_token_account = next_account_info(account_info_iter)?;
    let user_yes_token_account = next_account_info(account_info_iter)?;
    let user_no_token_account = next_account_info(account_info_iter)?;
    let pda_usd_token_account = next_account_info(account_info_iter)?;
    let token_program_id = next_account_info(account_info_iter)?;

    let (pda, bump_seed) = Pubkey::find_program_address(&[b"betting"], program_id);
    if *pda_account.key != pda {
      return Err(BettingMarketError::InvalidPda.into());
    }

    let data_ptr = betting_market_data_account.data.borrow_mut().as_mut_ptr();

    for i in 0..100 {
      if payout_exists_at_index(data_ptr, i) {
        let (payout_user_account, payout_mint, payout_amount) = get_payout_at_index(data_ptr, i);
        if payout_user_account == *user_account.key {
          // pay out the user and clear the payout
          if payout_mint == *usd_token_mint.key {
            msg!("payout {}: transfer {} usd to user", i, payout_amount);
            invoke_signed(
              &spl_token::instruction::transfer(
                token_program_id.key,
                pda_usd_token_account.key,
                user_usd_token_account.key,
                pda_account.key,
                &[],
                payout_amount,
              )?,
              &[
                pda_usd_token_account.clone(),
                user_usd_token_account.clone(),
                pda_account.clone(),
                token_program_id.clone(),
              ],
              &[&[&b"betting"[..], &[bump_seed]]],
            )?;
          } else if payout_mint == *yes_token_mint.key {
            msg!("payout {}: mint {} yes to user", i, payout_amount);
            invoke_signed(
              &spl_token::instruction::mint_to(
                token_program_id.key,
                yes_token_mint.key,
                user_yes_token_account.key,
                pda_account.key,
                &[],
                payout_amount,
              )?,
              &[
                yes_token_mint.clone(),
                user_yes_token_account.clone(),
                pda_account.clone(),
                token_program_id.clone(),
              ],
              &[&[&b"betting"[..], &[bump_seed]]],
            )?;
          } else if payout_mint == *no_token_mint.key {
            msg!("payout {}: mint {} no to user", i, payout_amount);
            invoke_signed(
              &spl_token::instruction::mint_to(
                token_program_id.key,
                no_token_mint.key,
                user_no_token_account.key,
                pda_account.key,
                &[],
                payout_amount,
              )?,
              &[
                no_token_mint.clone(),
                user_no_token_account.clone(),
                pda_account.clone(),
                token_program_id.clone(),
              ],
              &[&[&b"betting"[..], &[bump_seed]]],
            )?;
          } else {
            msg!("payout {}: bad payout mint", i);
          }
          set_payout_at_index(data_ptr, i, NULL_PUBKEY, NULL_PUBKEY, 0);
        }
      }
    }

    let result = get_u8_at_ptr_offset(data_ptr, 1);
    if result == 1 || result == 2 {
      // unpack token account data
      let user_yes_token_account_data: TokenAccount =
        TokenAccount::unpack(&user_yes_token_account.data.borrow())?;
      let user_no_token_account_data: TokenAccount =
        TokenAccount::unpack(&user_no_token_account.data.borrow())?;
      let user_yes_token_amount = user_yes_token_account_data.amount;
      let user_no_token_amount = user_no_token_account_data.amount;
      msg!("yes token amount: {}", user_yes_token_amount);
      msg!("no token amount: {}", user_no_token_amount);
      // burn all yes and no tokens
      invoke(
        &spl_token::instruction::burn(
          token_program_id.key,
          user_yes_token_account.key,
          yes_token_mint.key,
          user_account.key,
          &[],
          user_yes_token_amount,
        )?,
        &[
          user_yes_token_account.clone(),
          yes_token_mint.clone(),
          user_account.clone(),
          token_program_id.clone(),
        ],
      )?;
      invoke(
        &spl_token::instruction::burn(
          token_program_id.key,
          user_no_token_account.key,
          no_token_mint.key,
          user_account.key,
          &[],
          user_no_token_amount,
        )?,
        &[
          user_no_token_account.clone(),
          no_token_mint.clone(),
          user_account.clone(),
          token_program_id.clone(),
        ],
      )?;
      // transfer usd amount based on result
      let mut usd_transfer_amount = 0;
      if result == 1 {
        msg!(
          "betting market yes wins, transferring {} usd",
          user_yes_token_amount
        );
        usd_transfer_amount = user_yes_token_amount * 100;
      } else if result == 2 {
        msg!(
          "betting market no wins, transferring {} usd",
          user_no_token_amount
        );
        usd_transfer_amount = user_no_token_amount * 100;
      }
      invoke_signed(
        &spl_token::instruction::transfer(
          token_program_id.key,
          pda_usd_token_account.key,
          user_usd_token_account.key,
          pda_account.key,
          &[],
          usd_transfer_amount,
        )?,
        &[
          pda_usd_token_account.clone(),
          user_usd_token_account.clone(),
          pda_account.clone(),
          token_program_id.clone(),
        ],
        &[&[&b"betting"[..], &[bump_seed]]],
      )?;
      msg!("transferred {} usd", usd_transfer_amount);
    } else {
      msg!("betting market not judged yet");
    }

    Ok(())
  }

  // mint tokens to user for free
  fn process_free_mint(
    accounts: &[AccountInfo],
    amount: u64,
    program_id: &Pubkey,
  ) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pda_account = next_account_info(account_info_iter)?;
    let desired_token_mint = next_account_info(account_info_iter)?;
    let user_desired_token_account = next_account_info(account_info_iter)?;
    let token_program_id = next_account_info(account_info_iter)?;

    let (pda, bump_seed) = Pubkey::find_program_address(&[b"betting"], program_id);
    if *pda_account.key != pda {
      return Err(BettingMarketError::InvalidPda.into());
    }

    msg!(
      "minting {} of {} to user token account {}",
      amount,
      desired_token_mint.key,
      user_desired_token_account.key
    );
    invoke_signed(
      &spl_token::instruction::mint_to(
        token_program_id.key,
        desired_token_mint.key,
        user_desired_token_account.key,
        pda_account.key,
        &[],
        amount,
      )?,
      &[
        desired_token_mint.clone(),
        user_desired_token_account.clone(),
        pda_account.clone(),
        token_program_id.clone(),
      ],
      &[&[&b"betting"[..], &[bump_seed]]],
    )?;

    Ok(())
  }

  // set the betting market result manually
  fn process_judge_betting_market_manually(accounts: &[AccountInfo], result: u64) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let betting_market_data_account = next_account_info(account_info_iter)?;

    let data_ptr = betting_market_data_account.data.borrow_mut().as_mut_ptr();

    let old_result = get_u8_at_ptr_offset(data_ptr, 1);
    msg!("old result: {}", old_result);
    set_u8_at_ptr_offset(data_ptr, 1, result.try_into().unwrap());
    let new_result = get_u8_at_ptr_offset(data_ptr, 1);
    msg!("new result: {}", new_result);

    Ok(())
  }

  // set the betting market result with from oracle
  fn process_judge_betting_market_oracle(accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let betting_market_data_account = next_account_info(account_info_iter)?;
    let pyth_price_account = next_account_info(account_info_iter)?;

    let pyth_price_account_data = &pyth_price_account.try_borrow_data()?;
    let pyth_price = pyth_client::cast::<pyth_client::Price>(pyth_price_account_data);
    let data_ptr = betting_market_data_account.data.borrow_mut().as_mut_ptr();

    msg!("oracle price: {}", pyth_price.agg.price);
    let strike_price = get_u64_at_ptr_offset(data_ptr, 98);
    msg!("strike price: {}", strike_price);
    let betting_market_result =
      if pyth_price.agg.price > (strike_price * 1_000_000_000).try_into().unwrap() {
        1
      } else {
        2
      };
    msg!("betting market result: {}", betting_market_result);

    set_u8_at_ptr_offset(data_ptr, 1, betting_market_result);

    Ok(())
  }

  // set the betting market strike price
  fn process_set_strike_price(accounts: &[AccountInfo], strike_price: u64) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let betting_market_data_account = next_account_info(account_info_iter)?;

    let data_ptr = betting_market_data_account.data.borrow_mut().as_mut_ptr();

    let old_strike_price = get_u64_at_ptr_offset(data_ptr, 98);
    msg!("old strike price: {}", old_strike_price);
    set_u64_at_ptr_offset(data_ptr, 98, strike_price);
    let new_strike_price = get_u64_at_ptr_offset(data_ptr, 98);
    msg!("new strike price: {}", new_strike_price);

    Ok(())
  }
}
