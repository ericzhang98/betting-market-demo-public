use solana_program::{
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack, Sealed},
  pubkey::Pubkey,
};

use std::convert::TryFrom;

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

// data: is initialized, yes token mint, no token mint, usd token account, strike_price, result {0,1,2}, judge
// data account owned by this program and gets passed into all functions
pub struct BettingMarket {
  pub is_initialized: bool,
  pub result: u8,
  pub yes_token_mint: Pubkey,
  pub no_token_mint: Pubkey,
  pub usd_token_account: Pubkey,
  pub strike_price: u64,
  pub judge: Pubkey,
}

impl Sealed for BettingMarket {}

impl IsInitialized for BettingMarket {
  fn is_initialized(&self) -> bool {
    self.is_initialized
  }
}

impl Pack for BettingMarket {
  const LEN: usize = 138;

  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    let src = array_ref![src, 0, BettingMarket::LEN];
    let (
      is_initialized,
      result,
      yes_token_mint,
      no_token_mint,
      usd_token_account,
      strike_price,
      judge,
    ) = array_refs![src, 1, 1, 32, 32, 32, 8, 32];
    let is_initialized = match is_initialized {
      [0] => false,
      [1] => true,
      _ => return Err(ProgramError::InvalidAccountData),
    };
    // convert from u8 le array to u32 array
    // let mut bid_amounts_u32: [u32; 101] = [0; 101];
    // let mut ask_amounts_u32: [u32; 101] = [0; 101];
    // for i in (0..404).step_by(4) {
    //   bid_amounts_u32[i / 4] =
    //     u32::from_le_bytes(<[u8; 4]>::try_from(&bid_amounts_u8[i..i + 4]).unwrap());
    //   ask_amounts_u32[i / 4] =
    //     u32::from_le_bytes(<[u8; 4]>::try_from(&ask_amounts_u8[i..i + 4]).unwrap());
    // }
    // convert from u8 le array to Pubkey array
    // let null_pub_key = Pubkey::new_from_array([0; 32]);
    // let mut bid_queues_pubkey: [Pubkey; 10] = [null_pub_key.clone(); 10];
    // let mut ask_queues_pubkey: [Pubkey; 10] = [null_pub_key.clone(); 10];
    // for i in (0..320).step_by(32) {
    //   bid_queues_pubkey[i / 32] =
    //     Pubkey::new_from_array(<[u8; 32]>::try_from(&bid_queues[i..i + 32]).unwrap());
    //   ask_queues_pubkey[i / 32] =
    //     Pubkey::new_from_array(<[u8; 32]>::try_from(&ask_queues[i..i + 32]).unwrap());
    // }

    Ok(BettingMarket {
      is_initialized,
      result: u8::from_le_bytes(*result),
      yes_token_mint: Pubkey::new_from_array(*yes_token_mint),
      no_token_mint: Pubkey::new_from_array(*no_token_mint),
      usd_token_account: Pubkey::new_from_array(*usd_token_account),
      strike_price: u64::from_le_bytes(*strike_price),
      judge: Pubkey::new_from_array(*judge),
    })
  }

  fn pack_into_slice(&self, dst: &mut [u8]) {
    let dst = array_mut_ref![dst, 0, BettingMarket::LEN];
    let (
      is_initialized_dst,
      result_dst,
      yes_token_mint_dst,
      no_token_mint_dst,
      usd_token_account_dst,
      strike_price_dst,
      judge_dst,
    ) = mut_array_refs![dst, 1, 1, 32, 32, 32, 8, 32];
    let BettingMarket {
      is_initialized,
      result,
      yes_token_mint,
      no_token_mint,
      usd_token_account,
      strike_price,
      judge,
    } = self;
    is_initialized_dst[0] = *is_initialized as u8;
    *result_dst = result.to_le_bytes();
    yes_token_mint_dst.copy_from_slice(yes_token_mint.as_ref());
    no_token_mint_dst.copy_from_slice(no_token_mint.as_ref());
    usd_token_account_dst.copy_from_slice(usd_token_account.as_ref());
    *strike_price_dst = strike_price.to_le_bytes();
    judge_dst.copy_from_slice(judge.as_ref());
    // for i in (0..404).step_by(4) {
    //   bid_amounts_dst[i..i + 4].copy_from_slice(&bid_amounts[i / 4].to_le_bytes());
    //   ask_amounts_dst[i..i + 4].copy_from_slice(&ask_amounts[i / 4].to_le_bytes());
    // }
    // for i in (0..320).step_by(32) {
    //   bid_queues_dst[i..i + 32].copy_from_slice(bid_queues[i / 32].as_ref());
    //   ask_queues_dst[i..i + 32].copy_from_slice(ask_queues[i / 32].as_ref());
    // }
  }
}

// betting market data layout
// 0..138 - betting market metadata
// 1000..1808 - [u64; 101] - buy amounts for yes price
// 2000..2808 - [u64; 101] - buy amounts for no price
// 10000..42320 - [[Pubkey; 10]; 101] - user accounts associated with price (fifo)
// 50000..58080 - [[u64; 10]; 101] - payout in usd instead of token bool {1, 2} (fifo)
// 60000..68080 - [[u64; 10]; 101] - amounts associated with price (fifo)
// 70000..73200 - [Pubkey; 100] - payout user acc
// 80000..83200 - [Pubkey; 100] - payout mint
// 90000..90800 - [u64; 100] - payout amount

pub const YES_BUY_AMOUNT_START_OFFSET: isize = 1000;
pub const NO_BUY_AMOUNT_START_OFFSET: isize = 2000;
pub const USER_ACCOUNTS_FOR_PRICE_START_OFFSET: isize = 10000;
pub const PAYOUT_IN_USD_FOR_PRICE_START_OFFSET: isize = 50000;
pub const PAYOUT_AMOUNTS_FOR_PRICE_START_OFFSET: isize = 60000;
pub const PAYOUT_USER_ACCOUNTS_OFFSET: isize = 70000;
pub const PAYOUT_MINTS_OFFSET: isize = 80000;
pub const PAYOUT_AMOUNTS_OFFSET: isize = 90000;

pub const PUBKEY_USIZE: usize = 32;
pub const U64_USIZE: usize = 8;
pub const PUBKEY_ISIZE: isize = 32;
pub const PUBKEY_10_ISIZE: isize = 320;
pub const U64_ISIZE: isize = 8;
pub const U64_10_ISIZE: isize = 80;

pub const NULL_PUBKEY: Pubkey = Pubkey::new_from_array([0; 32]);

pub fn get_pubkey_10_at_ptr_offset(data_ptr: *const u8, offset: isize) -> [Pubkey; 10] {
  let mut arr = [NULL_PUBKEY; 10];
  for i in 0..10 {
    arr[i] = get_pubkey_at_ptr_offset(data_ptr, offset + ((i * PUBKEY_USIZE) as isize));
  }
  arr
}

pub fn get_pubkey_at_ptr_offset(data_ptr: *const u8, offset: isize) -> Pubkey {
  unsafe { get_pubkey_at_ptr(data_ptr.offset(offset)) }
}

pub fn get_pubkey_at_ptr(pubkey_ptr: *const u8) -> Pubkey {
  unsafe {
    let pubkey_slice = std::slice::from_raw_parts(pubkey_ptr, PUBKEY_USIZE);
    let pubkey_32_bytes = <[u8; PUBKEY_USIZE]>::try_from(pubkey_slice).unwrap();
    let pubkey = Pubkey::new_from_array(pubkey_32_bytes);
    pubkey
  }
}

pub fn set_pubkey_10_at_ptr_offset(data_ptr: *mut u8, offset: isize, pubkey_10: [Pubkey; 10]) {
  for i in 0..10 {
    set_pubkey_at_ptr_offset(
      data_ptr,
      offset + ((i * PUBKEY_USIZE) as isize),
      pubkey_10[i],
    );
  }
}

pub fn set_pubkey_at_ptr_offset(data_ptr: *mut u8, offset: isize, pubkey: Pubkey) {
  unsafe {
    set_pubkey_at_ptr(data_ptr.offset(offset), pubkey);
  }
}

pub fn set_pubkey_at_ptr(pubkey_ptr: *mut u8, pubkey: Pubkey) {
  unsafe {
    let mutable_pubkey_slice = std::slice::from_raw_parts_mut(pubkey_ptr, PUBKEY_USIZE);
    mutable_pubkey_slice.copy_from_slice(pubkey.as_ref());
  }
}

pub fn get_u64_10_at_ptr_offset(data_ptr: *const u8, offset: isize) -> [u64; 10] {
  let mut arr = [0; 10];
  for i in 0..10 {
    arr[i] = get_u64_at_ptr_offset(data_ptr, offset + ((i * U64_USIZE) as isize));
  }
  arr
}

pub fn get_u64_at_ptr_offset(data_ptr: *const u8, offset: isize) -> u64 {
  unsafe { get_u64_at_ptr(data_ptr.offset(offset)) }
}

pub fn get_u64_at_ptr(num_ptr: *const u8) -> u64 {
  unsafe {
    let num_slice = std::slice::from_raw_parts(num_ptr, U64_USIZE);
    let num_8_bytes = <[u8; U64_USIZE]>::try_from(num_slice).unwrap();
    let num = u64::from_le_bytes(num_8_bytes);
    num
  }
}

pub fn set_u64_10_at_ptr_offset(data_ptr: *mut u8, offset: isize, num_10: [u64; 10]) {
  for i in 0..10 {
    set_u64_at_ptr_offset(data_ptr, offset + ((i * U64_USIZE) as isize), num_10[i]);
  }
}

pub fn set_u64_at_ptr_offset(data_ptr: *mut u8, offset: isize, num: u64) {
  unsafe {
    set_u64_at_ptr(data_ptr.offset(offset), num);
  }
}

pub fn set_u64_at_ptr(num_ptr: *mut u8, num: u64) {
  unsafe {
    let mutable_num_slice = std::slice::from_raw_parts_mut(num_ptr, U64_USIZE);
    mutable_num_slice.copy_from_slice(num.to_le_bytes().as_ref());
  }
}

pub fn get_u8_at_ptr_offset(data_ptr: *const u8, offset: isize) -> u8 {
  unsafe { get_u8_at_ptr(data_ptr.offset(offset)) }
}

pub fn get_u8_at_ptr(u8_ptr: *const u8) -> u8 {
  unsafe { *u8_ptr }
}

pub fn set_u8_at_ptr_offset(data_ptr: *mut u8, offset: isize, value: u8) {
  unsafe {
    set_u8_at_ptr(data_ptr.offset(offset), value);
  }
}

pub fn set_u8_at_ptr(u8_ptr: *mut u8, value: u8) {
  unsafe {
    *u8_ptr = value;
  }
}

pub fn payout_exists_at_index(data_ptr: *mut u8, index: isize) -> bool {
  let payout_user_account_offset = PAYOUT_USER_ACCOUNTS_OFFSET + PUBKEY_ISIZE * (index as isize);
  if get_pubkey_at_ptr_offset(data_ptr, payout_user_account_offset) == NULL_PUBKEY {
    false
  } else {
    true
  }
}

pub fn get_payout_at_index(data_ptr: *mut u8, index: isize) -> (Pubkey, Pubkey, u64) {
  let payout_user_account_offset = PAYOUT_USER_ACCOUNTS_OFFSET + PUBKEY_ISIZE * (index as isize);
  let payout_mints_offset = PAYOUT_MINTS_OFFSET + PUBKEY_ISIZE * (index as isize);
  let payout_amounts_offset = PAYOUT_AMOUNTS_OFFSET + U64_ISIZE * (index as isize);
  (
    get_pubkey_at_ptr_offset(data_ptr, payout_user_account_offset),
    get_pubkey_at_ptr_offset(data_ptr, payout_mints_offset),
    get_u64_at_ptr_offset(data_ptr, payout_amounts_offset),
  )
}

pub fn set_payout_at_index(
  data_ptr: *mut u8,
  index: isize,
  payout_user_account: Pubkey,
  payout_mint: Pubkey,
  payout_amount: u64,
) {
  let payout_user_account_offset = PAYOUT_USER_ACCOUNTS_OFFSET + PUBKEY_ISIZE * (index as isize);
  let payout_mints_offset = PAYOUT_MINTS_OFFSET + PUBKEY_ISIZE * (index as isize);
  let payout_amounts_offset = PAYOUT_AMOUNTS_OFFSET + U64_ISIZE * (index as isize);
  set_pubkey_at_ptr_offset(data_ptr, payout_user_account_offset, payout_user_account);
  set_pubkey_at_ptr_offset(data_ptr, payout_mints_offset, payout_mint);
  set_u64_at_ptr_offset(data_ptr, payout_amounts_offset, payout_amount);
}
