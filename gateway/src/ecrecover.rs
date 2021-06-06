use ethabi::Address;
use primitive_types::H256;

mod costs {
    pub(super) const ECRECOVER_BASE: u64 = 3_000;
}

mod consts {
    pub(super) const INPUT_LEN: usize = 128;
}

/// See: https://ethereum.github.io/yellowpaper/paper.pdf
/// See: https://docs.soliditylang.org/en/develop/units-and-global-variables.html#mathematical-and-cryptographic-functions
/// See: https://etherscan.io/address/0000000000000000000000000000000000000001
// Quite a few library methods rely on this and that should be changed. This
// should only be for precompiles.
pub(crate) fn ecrecover(hash: H256, signature: &[u8]) -> Result<Address, ()> {
    use sha3::Digest;
    assert_eq!(signature.len(), 65);

    let hash = secp256k1::Message::parse_slice(hash.as_bytes()).unwrap();
    let v = signature[64];
    let signature = secp256k1::Signature::parse_slice(&signature[0..64]).unwrap();
    let bit = match v {
        0..=26 => v,
        _ => v - 27,
    };

    if let Ok(recovery_id) = secp256k1::RecoveryId::parse(bit) {
        if let Ok(public_key) = secp256k1::recover(&hash, &signature, &recovery_id) {
            // recover returns a 65-byte key, but addresses come from the raw 64-byte key
            let r = sha3::Keccak256::digest(&public_key.serialize()[1..]);
            return Ok(Address::from_slice(&r[12..]));
        }
    }

    Err(())
}
