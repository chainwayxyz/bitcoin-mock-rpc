//! # Utilities
//!
//! This crate includes helper utilities.

use crate::ledger::errors::LedgerError;
use bitcoin::{
    consensus::Encodable,
    hashes::{sha256, Hash},
    TxMerkleNode,
};
use rs_merkle::{Hasher, MerkleTree};

/// Bitcoin merkle root hashing algorithm.
#[derive(Clone)]
pub struct Hash256 {}
impl Hasher for Hash256 {
    type Hash = [u8; 32];

    /// Double SHA256 for merkle root calculation.
    fn hash(data: &[u8]) -> [u8; 32] {
        sha256::Hash::hash(&sha256::Hash::hash(data).to_byte_array()).to_byte_array()
    }
}

/// Calculates given inputs merkle root. If input number is odd, last input will
/// be added to the list again.
///
/// This merkle root calculator is useful for TXID and wTXID merkle roots.
///
/// # Parameters
///
/// - `inputs`: Input values.
pub fn calculate_merkle_root<T>(inputs: Vec<T>) -> Result<TxMerkleNode, LedgerError>
where
    T: Encodable,
{
    let mut leaves: Vec<_> = inputs
        .iter()
        .map(|input| {
            let mut hex: Vec<u8> = Vec::new();
            input.consensus_encode(&mut hex).unwrap();

            let mut arr: [u8; 32] = [32; 32];
            arr[..hex.len()].copy_from_slice(&hex[..]);

            arr
        })
        .collect();

    // If there are odd numbered transactions, we must concatenate and hash
    // with itself. Hashing is done by the MerkleTree library. We only need
    // to add an extra TXID/wTXID to the list.
    let len = leaves.len();
    if len % 2 == 1 {
        leaves.push(leaves[len - 1]);
    }

    let merkle_tree = MerkleTree::<Hash256>::from_leaves(leaves.as_slice());

    let root = match merkle_tree.root() {
        Some(r) => r,
        None => return Ok(TxMerkleNode::all_zeros()),
    };

    let hash = match Hash::from_slice(root.as_slice()) {
        Ok(h) => h,
        Err(e) => {
            return Err(LedgerError::Transaction(format!(
                "Couldn't convert root {:?} to hash: {}",
                root, e
            )))
        }
    };

    Ok(TxMerkleNode::from_raw_hash(hash))
}

/// Converts a hex string to an [u8] array. Encoding is done by converting hex
/// value to digit value and packing 2 digits together.
///
/// # Parameters
///
/// - `hex`: Hex encoded string with no prefixes nor suffixes
/// - `output`: Mutable array that will hold encoded data
///
/// # Examples
///
/// ```ignore
/// let mut hex: [u8; 1] = [0; 1];
/// hex_to_array("FF", &mut hex);
/// assert_eq!(hex, [255]);
/// ```
///
/// # Panics
///
/// Will panic if input `hex` length is more than 2 times of `output` length.
pub fn hex_to_array(hex: &str, output: &mut [u8]) {
    // Clean output.
    for item in &mut *output {
        *item = 0;
    }

    let len = hex.len();

    hex.chars().enumerate().for_each(|(idx, char)| {
        output[idx / 2] += if idx % 2 == 0 && idx + 1 != len {
            char.to_digit(16).unwrap() as u8 * 16
        } else {
            char.to_digit(16).unwrap() as u8
        };
    });
}

#[cfg(test)]
mod tests {
    use bitcoin::{hashes::sha256d::Hash, TxMerkleNode, Txid};
    use std::str::FromStr;

    #[test]
    fn hex_to_array() {
        let mut hex: [u8; 1] = [0; 1];
        super::hex_to_array("F", &mut hex);
        assert_eq!(hex, [15]);

        let mut hex: [u8; 2] = [0; 2];
        super::hex_to_array("1234", &mut hex);
        assert_eq!(hex, [18, 52]);

        let mut hex: [u8; 2] = [0; 2];
        super::hex_to_array("ABCD", &mut hex);
        assert_eq!(hex, [171, 205]);

        let mut hex: [u8; 4] = [0; 4];
        super::hex_to_array("B00B1E5", &mut hex);
        assert_eq!(hex, [176, 11, 30, 5]);

        // Dirty input data.
        let mut hex: [u8; 4] = [0x45; 4];
        super::hex_to_array("B00B1E5", &mut hex);
        assert_eq!(hex, [176, 11, 30, 5]);
    }

    #[test]
    fn calculate_merkle_root_even_numbered() {
        let txids = [
            Txid::from_str("8c14f0db3df150123e6f3dbbf30f8b955a8249b62ac1d1ff16284aefa3d06d87")
                .unwrap(),
            Txid::from_str("fff2525b8931402dd09222c50775608f75787bd2b87e56995a7bdd30f79702c4")
                .unwrap(),
            Txid::from_str("6359f0868171b1d194cbee1af2f16ea598ae8fad666d9b012c8ed2b79a236ec4")
                .unwrap(),
            Txid::from_str("e9a66845e05d5abc0ad04ec80f774a7e585c6e8db975962d069a522137b80c1d")
                .unwrap(),
        ];

        let merkle_root = super::calculate_merkle_root(txids.to_vec().clone()).unwrap();

        assert_eq!(
            TxMerkleNode::from_raw_hash(
                Hash::from_str("f3e94742aca4b5ef85488dc37c06c3282295ffec960994b2c0d5ac2a25a95766")
                    .unwrap()
            ),
            merkle_root
        );
    }

    #[test]
    fn calculate_merkle_root_odd_numbered() {
        let txids = [
            Txid::from_str("ff4861ebd4709ba120b8cb418385cc3ff5184a917fb91f7dff03ecb521ca192e")
                .unwrap(),
            Txid::from_str("97a3dd8b297a0e46f274036a5f78b43c16286e3a34271ca6be7a1b51bba16a71")
                .unwrap(),
            Txid::from_str("02d08b73223be820351d0edc2a40046e320efacb3ab665dedf36ae178086565e")
                .unwrap(),
        ];

        let merkle_root = super::calculate_merkle_root(txids.to_vec().clone()).unwrap();

        assert_eq!(
            TxMerkleNode::from_raw_hash(
                Hash::from_str("8c1c8ff245b6d8bbbde4e8fc6686c6b57d2911fc5ec9743658f0f04868377df3")
                    .unwrap()
            ),
            merkle_root
        );
    }
}
