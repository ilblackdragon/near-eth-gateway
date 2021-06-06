use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::Base64VecU8;
use near_sdk::{env, near_bindgen, Gas, PanicOnDefault, Promise};
use primitive_types::U256;

pub use crate::meta_parsing::{near_erc712_domain, prepare_meta_call_args};
pub use crate::types::{u256_to_arr, InternalMetaCallArgs, MetaCallArgs};
use crate::types::{Address, RawAddress, RawU256};

mod ecrecover;
mod meta_parsing;
mod types;

near_sdk::setup_alloc!();

const CHAIN_ID: u64 = 1;

const CODE: &[u8] = include_bytes!("../../res/proxy.wasm");

const TGAS: Gas = 1_000_000_000_000;
const GAS_FOR_PROXY: Gas = 10 * TGAS;

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    nonces: LookupMap<RawAddress, RawU256>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            nonces: LookupMap::new(b"n".to_vec()),
        }
    }

    /// Parses given message into meta call arguments.
    /// Asserts that all the information is correct, like chain_id, destination contract and nonce.
    fn parse_message(&mut self, message: Base64VecU8) -> InternalMetaCallArgs {
        let domain_separator = crate::meta_parsing::near_erc712_domain(U256::from(CHAIN_ID));
        let args = crate::meta_parsing::parse_meta_call(
            &domain_separator,
            &env::current_account_id().into_bytes(),
            message.0,
        )
        .expect("ERR_META_TX_PARSE");
        let nonce = self
            .nonces
            .get(&args.sender.0)
            .map(|value| U256::from(value))
            .unwrap_or_default();
        assert_eq!(args.nonce, nonce, "ERR_INCORRECT_NONCE");
        self.nonces
            .insert(&args.sender.0, &u256_to_arr(&(nonce + 1)));
        args
    }

    #[payable]
    pub fn create(&mut self, message: Base64VecU8) -> Promise {
        let args = self.parse_message(message);
        let account_id = format!("{}.{}", hex::encode(args.sender), env::current_account_id());
        Promise::new(account_id)
            .create_account()
            .deploy_contract(CODE.to_vec())
            .transfer(env::attached_deposit())
    }

    pub fn proxy(&mut self, message: Base64VecU8) -> Promise {
        let args = self.parse_message(message);
        let mut transfer_args = vec![0u8; 16 + args.contract_address.len()];
        transfer_args[..16].copy_from_slice(&args.value.to_le_bytes());
        transfer_args[16..].copy_from_slice(args.contract_address.as_bytes());
        let account_id = format!("{}.{}", hex::encode(args.sender), env::current_account_id());
        Promise::new(account_id).function_call(
            "transfer".as_bytes().to_vec(),
            transfer_args,
            0,
            TGAS * 10,
            // env::prepaid_gas() - GAS_FOR_PROXY,
        )
    }

    // pub fn update(&self, message: Base64VecU8) -> Promise {
    //     Promise::new(account_id).function_call("update", )
    // }
}
