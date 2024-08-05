//! # Block Related Ledger Operations

use super::errors::LedgerError;
use super::Ledger;
use bitcoin::block::{Header, Version};
use bitcoin::consensus::{Decodable, Encodable};
use bitcoin::hashes::{sha256, Hash};
use bitcoin::{Block, BlockHash, CompactTarget, Transaction, TxMerkleNode, Txid};
use rs_merkle::{Hasher, MerkleTree};
use rusqlite::params;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

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

impl Ledger {
    /// Mines current transactions that are in mempool to a block.
    ///
    /// # Panics
    ///
    /// Will panic if there was a problem writing data to ledger.
    pub fn mine_block(&self) -> Result<BlockHash, LedgerError> {
        let transactions = self.get_mempool_transactions();
        let block = self.create_block(transactions)?;

        self.clean_mempool();
        self.add_block(block)
    }

    /// Creates a block using given transactions.
    pub fn create_block(&self, transactions: Vec<Transaction>) -> Result<Block, LedgerError> {
        let prev_block_height = self.get_block_height()?;
        let prev_block_time = self.get_block_time(prev_block_height)?;

        let prev_blockhash = match self.get_block_with_height(prev_block_height) {
            Ok(b) => b.block_hash(),
            Err(e) => {
                if prev_block_height >= 1 {
                    return Err(LedgerError::Block(format!(
                        "Couldn't get previous block hash with height {}: {}",
                        prev_block_height, e
                    )));
                }

                BlockHash::all_zeros()
            }
        };

        let txids: Vec<Txid> = transactions.iter().map(|tx| tx.compute_txid()).collect();
        let merkle_root = self.calculate_merkle_root(txids)?;

        Ok(Block {
            header: Header {
                version: Version::TWO,
                prev_blockhash,
                merkle_root,
                time: prev_block_time + (10 * 60),
                bits: CompactTarget::from_consensus(0x20FFFFFF),
                nonce: 0,
            },
            txdata: transactions,
        })
    }

    /// Adds a block to ledger.
    ///
    /// Uses current block height and time to calculate next block height and
    /// time. Previous height + 1 is used for height while previous time + 10
    /// minutes is used for time.
    ///
    /// # Panics
    ///
    /// Will panic if there was a problem writing data to ledger.
    fn add_block(&self, block: Block) -> Result<BlockHash, LedgerError> {
        let prev_block_height = self.get_block_height()?;
        let prev_block_time = self.get_block_time(prev_block_height)?;

        let current_block_height = prev_block_height + 1;
        let current_block_time = prev_block_time + (10 * 60);

        let mut hash: Vec<u8> = Vec::new();
        block.block_hash().consensus_encode(&mut hash).unwrap();

        let mut body: Vec<u8> = Vec::new();
        if let Err(e) = block.consensus_encode(&mut body) {
            return Err(LedgerError::Block(format!("Couldn't encode block: {}", e)));
        };

        if let Err(e) = self.database.lock().unwrap().execute(
            "INSERT INTO blocks (height, time, hash, body) VALUES (?1, ?2, ?3, ?4)",
            params![current_block_height, current_block_time, hash, body],
        ) {
            return Err(LedgerError::Block(format!(
                "Couldn't add block {:?} to ledger: {}",
                block, e
            )));
        };

        Ok(block.block_hash())
    }
    /// Returns a block with `height` from ledger.
    pub fn get_block_with_height(&self, height: u32) -> Result<Block, LedgerError> {
        let body = match self.database.lock().unwrap().query_row(
            "SELECT body FROM blocks WHERE height = ?1",
            params![height],
            |row| Ok(row.get::<_, Vec<u8>>(0)),
        ) {
            Ok(qr) => qr,
            Err(e) => {
                return Err(LedgerError::Block(format!(
                    "Couldn't find any block with block height {}: {}",
                    height, e
                )))
            }
        };
        // Genesis block will also return a database error. Ignore that.
        let body = match body {
            Ok(b) => b,
            Err(_) => Vec::new(),
        };

        match Block::consensus_decode(&mut body.as_slice()) {
            Ok(block) => Ok(block),
            Err(e) => Err(LedgerError::Block(format!(
                "Internal error while reading block from ledger: {}",
                e
            ))),
        }
    }
    /// Returns a block with `height` from ledger.
    pub fn get_block_with_hash(&self, hash: BlockHash) -> Result<Block, LedgerError> {
        let mut encoded_hash: Vec<u8> = Vec::new();
        hash.consensus_encode(&mut encoded_hash).unwrap();

        let qr = match self.database.lock().unwrap().query_row(
            "SELECT body FROM blocks WHERE hash = ?1",
            params![encoded_hash],
            |row| Ok(row.get::<_, Vec<u8>>(0).unwrap()),
        ) {
            Ok(qr) => qr,
            Err(e) => {
                return Err(LedgerError::Block(format!(
                    "Couldn't find any block with block height {}: {}",
                    hash, e
                )))
            }
        };

        match Block::consensus_decode(&mut qr.as_slice()) {
            Ok(block) => Ok(block),
            Err(e) => Err(LedgerError::Block(format!(
                "Internal error while reading block from ledger: {}",
                e
            ))),
        }
    }

    fn calculate_merkle_root(&self, txids: Vec<Txid>) -> Result<TxMerkleNode, LedgerError> {
        let leaves: Vec<_> = txids
            .iter()
            .map(|txid| {
                let mut hex: Vec<u8> = Vec::new();
                txid.consensus_encode(&mut hex).unwrap();

                let mut arr: [u8; 32] = [32; 32];
                for i in 0..hex.len() {
                    arr[i] = hex[i];
                }

                arr
            })
            .collect();

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

    /// Returns current block height.
    ///
    /// # Panics
    ///
    /// Will panic if cannot get height from database.
    pub fn get_block_height(&self) -> Result<u32, LedgerError> {
        match self.database.lock().unwrap().query_row(
            "SELECT height FROM blocks ORDER BY height DESC LIMIT 1",
            params![],
            |row| {
                let body = row.get::<_, i64>(0).unwrap();

                Ok(body as u32)
            },
        ) {
            Ok(h) => Ok(h),
            Err(e) => Err(LedgerError::Block(format!(
                "Couldn't read block height from ledger: {}",
                e
            ))),
        }
    }

    /// Gets all the transactions that are in the mempool.
    ///
    /// # Panics
    ///
    /// Will panic if there is a problem with database.
    pub fn get_mempool_transactions(&self) -> Vec<Transaction> {
        // If `txids` is not calculated in a separate scope, there will be a
        // deadlock. Because `get_transaction()` will also try to lock the
        // mutex. So, we do this operation first and unlock mutex for the next
        // call.
        let txids: Vec<Txid> = {
            let db = self.database.lock().unwrap();
            let mut stmt = db.prepare("SELECT (txid) FROM mempool").unwrap();
            let tx_iter = stmt
                .query_map([], |row| {
                    let body: String = row.get(0).unwrap();
                    Ok(Txid::from_str(&body).unwrap())
                })
                .unwrap();
            tx_iter.map(|txid| txid.unwrap()).collect()
        };

        txids
            .iter()
            .map(|txid| self.get_transaction(*txid).unwrap())
            .collect::<Vec<Transaction>>()
    }
    /// Adds a transactions to the mempool.
    ///
    /// # Panics
    ///
    /// Will panic if there is a problem with database.
    pub fn add_mempool_transaction(&self, txid: Txid) -> Result<(), LedgerError> {
        match self.database.lock().unwrap().execute(
            "INSERT INTO mempool (txid) VALUES (?1)",
            params![txid.to_string()],
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(LedgerError::Transaction(format!(
                "Couldn't add transaction with txid {} to mempool: {}",
                txid, e
            ))),
        }
    }

    /// Gets a mempool transaction, if it's in the mempool.
    ///
    /// # Panics
    ///
    /// Will panic if there is a problem with database.
    pub fn get_mempool_transaction(&self, txid: Txid) -> Option<Transaction> {
        let mempool_txs = self.get_mempool_transactions();

        mempool_txs
            .iter()
            .find(|tx| {
                if tx.compute_txid() == txid {
                    true
                } else {
                    false
                }
            })
            .cloned()
    }

    /// Cleans up mempool. This should only be called when transactions are
    /// mined.
    ///
    /// # Panics
    ///
    /// Will panic if there is a problem with database.
    pub fn clean_mempool(&self) {
        self.database
            .lock()
            .unwrap()
            .execute("DELETE FROM mempool", params![])
            .unwrap();
    }

    /// Gets `block_height`'th block time, in UNIX format.
    ///
    /// # Panics
    ///
    /// Will panic if there is a problem with database.
    pub fn get_block_time(&self, block_height: u32) -> Result<u32, LedgerError> {
        // Use current time for genesis block.
        if block_height == 1 {
            return Ok(SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32);
        }

        match self.database.lock().unwrap().query_row(
            "SELECT time FROM blocks WHERE height = ?1",
            params![block_height],
            |row| Ok(row.get::<_, i64>(0).unwrap() as u32),
        ) {
            Ok(time) => Ok(time),
            Err(e) => Err(LedgerError::Block(format!(
                "Invalid block number {}: {}",
                block_height, e
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ledger::Ledger;
    use bitcoin::{hashes::sha256d::Hash, Amount, ScriptBuf, Transaction, TxMerkleNode, Txid};
    use std::str::FromStr;

    #[test]
    fn mine_blocks_and_mempool() {
        let ledger = Ledger::new("mine_blocks_and_mempool");

        let current_height = ledger.get_block_height().unwrap();
        assert_eq!(current_height, 0);

        let tx = ledger.create_transaction(vec![], vec![]);
        ledger.add_transaction_unconditionally(tx.clone()).unwrap();

        assert_eq!(ledger.get_mempool_transactions().len(), 1);
        assert_eq!(
            ledger.get_mempool_transaction(tx.compute_txid()).unwrap(),
            tx
        );

        ledger.mine_block().unwrap();

        let current_height = ledger.get_block_height().unwrap();
        assert_eq!(current_height, 1);

        assert_eq!(ledger.get_mempool_transactions().len(), 0);
        if let Some(_) = ledger.get_mempool_transaction(tx.compute_txid()) {
            assert!(false);
        }
    }

    #[test]
    fn create_add_get_block_with_height() {
        let ledger = Ledger::new("create_add_get_block_with_height");

        ledger.mine_block().unwrap();
        ledger.mine_block().unwrap();

        let mut transactions: Vec<Transaction> = Vec::new();
        for i in 0..0x45 {
            let txout = ledger.create_txout(Amount::from_sat(0x45 * i), ScriptBuf::new());
            let tx = ledger.create_transaction(vec![], vec![txout]);

            transactions.push(tx);
        }

        let block = ledger.create_block(transactions).unwrap();

        ledger.add_block(block.clone()).unwrap();
        let block_height = ledger.get_block_height().unwrap();

        let read_block = ledger.get_block_with_height(block_height).unwrap();

        assert_eq!(block, read_block);
    }

    #[test]
    fn create_add_get_block_with_hash() {
        let ledger = Ledger::new("create_add_get_block_with_hash");
        ledger.mine_block().unwrap();

        let mut transactions: Vec<Transaction> = Vec::new();
        for i in 0..0x1F {
            let txout = ledger.create_txout(Amount::from_sat(0x1F * i), ScriptBuf::new());
            let tx = ledger.create_transaction(vec![], vec![txout]);

            transactions.push(tx);
        }

        let block = ledger.create_block(transactions).unwrap();
        let block_hash = block.block_hash();

        ledger.add_block(block.clone()).unwrap();

        let read_block = ledger.get_block_with_hash(block_hash).unwrap();

        assert_eq!(block, read_block);
    }

    #[test]
    fn merkle_tree() {
        let ledger = Ledger::new("merkle_tree");

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

        let merkle_root = ledger
            .calculate_merkle_root(txids.to_vec().clone())
            .unwrap();

        assert_eq!(
            TxMerkleNode::from_raw_hash(
                Hash::from_str("f3e94742aca4b5ef85488dc37c06c3282295ffec960994b2c0d5ac2a25a95766")
                    .unwrap()
            ),
            merkle_root
        );
    }
}
