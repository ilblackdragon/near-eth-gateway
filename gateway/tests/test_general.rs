use near_sdk_sim::{call, deploy, init_simulator, to_yocto, ExecutionResult, UserAccount};

use ethabi::Address;
use gateway::{
    near_erc712_domain, prepare_meta_call_args, u256_to_arr, ContractContract as Contract,
    InternalMetaCallArgs, MetaCallArgs,
};
use near_sdk::json_types::Base64VecU8;
use near_sdk::{Balance, Gas};
use near_sdk_sim::borsh::BorshSerialize;
use near_sdk_sim::near_crypto::{InMemorySigner, KeyType, PublicKey, Signature, Signer};
use near_sdk_sim::runtime::init_runtime;
use near_sdk_sim::transaction::ExecutionStatus::{SuccessReceiptId, SuccessValue};
use primitive_types::{H256, U256};
use sha3::Digest;
use std::cell::RefCell;
use std::rc::Rc;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    GATEWAY_WASM => "../res/gateway.wasm"
}

const TGAS: Gas = 1_000_000_000_000;

pub fn encode_meta_call_function_args(
    signer: &dyn Signer,
    chain_id: u64,
    nonce: U256,
    fee_amount: Balance,
    fee_address: String,
    contract_address: String,
    value: Balance,
    method_def: &str,
    args: Vec<u8>,
) -> Vec<u8> {
    let domain_separator = near_erc712_domain(U256::from(chain_id));
    let (msg, _) = match prepare_meta_call_args(
        &domain_separator,
        "gateway".as_bytes(),
        method_def.to_string(),
        &InternalMetaCallArgs {
            sender: Address::zero(),
            nonce,
            fee_amount,
            fee_address: fee_address.clone(),
            contract_address: contract_address.clone(),
            value,
            input: args.clone(),
        },
    ) {
        Ok(x) => x,
        Err(_) => panic!("Failed to prepare"),
    };
    match signer.sign(&msg) {
        Signature::ED25519(_) => panic!("Wrong Signer"),
        Signature::SECP256K1(sig) => {
            let array = Into::<[u8; 65]>::into(sig.clone()).to_vec();
            let mut signature = [0u8; 64];
            signature.copy_from_slice(&array[..64]);
            MetaCallArgs {
                signature,
                // Add 27 to align eth-sig-util signature format
                v: array[64] + 27,
                nonce: u256_to_arr(&nonce),
                fee_amount: u256_to_arr(&U256::from(fee_amount)),
                fee_address,
                contract_address,
                value: u256_to_arr(&U256::from(value)),
                method: method_def.to_string(),
                args,
            }
            .try_to_vec()
            .expect("Failed to serialize")
        }
    }
}

pub fn public_key_to_address(public_key: PublicKey) -> Address {
    match public_key {
        PublicKey::ED25519(_) => panic!("Wrong PublicKey"),
        PublicKey::SECP256K1(pubkey) => {
            let pk: [u8; 64] = pubkey.into();
            let bytes = H256::from_slice(sha3::Keccak256::digest(&pk.to_vec()).as_slice());
            let mut result = Address::zero();
            result.as_bytes_mut().copy_from_slice(&bytes[12..]);
            result
        }
    }
}

struct Wallet {
    signer: InMemorySigner,
    nonce: U256,
    chain_id: u64,
    pub public_key: Address,
}

impl Wallet {
    pub fn new() -> Self {
        let signer = InMemorySigner::from_seed("doesnt", KeyType::SECP256K1, "a");
        Self {
            public_key: public_key_to_address(signer.public_key.clone()),
            signer,
            nonce: U256::zero(),
            chain_id: 1,
        }
    }

    pub fn message(&mut self, receiver_id: &str, value: Balance, method_def: &str) -> Base64VecU8 {
        let result = encode_meta_call_function_args(
            &self.signer,
            self.chain_id,
            self.nonce,
            5,
            "token".to_string(),
            receiver_id.to_string(),
            value,
            method_def,
            vec![],
        );
        self.nonce += U256::one();
        Base64VecU8(result)
    }
}

fn assert_success(result: ExecutionResult) {
    match result.is_ok() {
        true => {}
        false => {
            for promise in result.promise_results() {
                println!("{:?}", promise);
            }
            result.assert_success();
        }
    }
}

#[test]
fn test_basics() {
    let root = init_simulator(None);
    let user2 = root.create_user("user2".to_string(), to_yocto("1"));
    let gateway = deploy!(contract: Contract, contract_id: "test".to_string(), bytes: &GATEWAY_WASM, signer_account: root, init_method: new());

    let mut wallet = Wallet::new();
    let message = wallet.message("", 0, "create()");

    call!(
        root,
        gateway.create(message),
        deposit = 165630000000000000000000
    )
    .assert_success();

    let new_account = format!("{}.test", hex::encode(&wallet.public_key));
    root.transfer(new_account.clone(), to_yocto("2"));

    // check that new account exists.
    let acc = root.borrow_runtime().view_account(&new_account).unwrap();
    println!("{:?}", acc);

    let message = wallet.message("user2", to_yocto("1"), "transfer()");
    assert_success(call!(root, gateway.proxy(message), gas = 100 * TGAS));
    assert_eq!(
        root.borrow_runtime().view_account("user2").unwrap().amount,
        to_yocto("2")
    );
}
