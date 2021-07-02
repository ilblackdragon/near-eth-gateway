use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, Balance};
use primitive_types::{H160, U256};

#[cfg(not(target_arch = "wasm32"))]
use sha3::Digest;

pub type RawAddress = [u8; 20];
pub type RawU256 = [u8; 32];

/// See: https://ethereum-magicians.org/t/increasing-address-size-from-20-to-32-bytes/5485
pub type Address = H160;

/// Incoming argument encoding.
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct MetaCallArgs {
    pub signature: [u8; 64],
    pub v: u8,
    pub nonce: RawU256,
    pub fee_amount: RawU256,
    pub fee_address: String,
    pub contract_address: String,
    pub value: RawU256,
    pub method: String,
    pub args: Vec<u8>,
}

/// Internal args format for meta call.
#[derive(Debug)]
pub struct InternalMetaCallArgs {
    pub sender: Address,
    pub nonce: U256,
    pub fee_amount: Balance,
    pub fee_address: String,
    pub contract_address: String,
    pub method_name: String,
    pub value: Balance,
    pub args: Vec<u8>,
}

pub fn u256_to_arr(value: &U256) -> [u8; 32] {
    let mut result = [0u8; 32];
    value.to_big_endian(&mut result);
    result
}

pub fn arr_to_u256(value: &[u8]) -> RawU256 {
    let mut result = RawU256::default();
    result.copy_from_slice(&value);
    result
}

#[cfg(target_arch = "wasm32")]
pub fn keccak256(data: &[u8]) -> Vec<u8> {
    env::keccak256(data)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn keccak256(data: &[u8]) -> Vec<u8> {
    sha3::Keccak256::digest(data).as_slice().to_vec()
}
