//! # Block Related Ledger Operations

use super::errors::LedgerError;
use super::Ledger;
use bitcoin::block::{Header, Version};
use bitcoin::consensus::{Decodable, Encodable};
use bitcoin::hashes::Hash;
use bitcoin::{Block, BlockHash, CompactTarget, Transaction, TxMerkleNode, Txid};
use rs_merkle::algorithms::Sha256;
use rs_merkle::MerkleTree;
use rusqlite::params;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

impl Ledger {
    /// Adds a block to ledger.
    pub fn add_block(&self, block: Block) -> Result<(), LedgerError> {
        let current_block_height = self.get_block_height();

        let mut raw_body: Vec<u8> = Vec::new();
        if let Err(e) = block.consensus_encode(&mut raw_body) {
            return Err(LedgerError::Block(format!("Couldn't encode block: {}", e)));
        };

        if let Err(e) = self.database.lock().unwrap().execute(
            "INSERT INTO tmpblocks (block_height, raw_body) VALUES (?1, ?2)",
            params![current_block_height, raw_body],
        ) {
            return Err(LedgerError::Block(format!(
                "Couldn't add block {:?} to ledger: {}",
                block, e
            )));
        };

        Ok(())
    }
    /// Returns a block with `height` from ledger.
    pub fn get_block_with_height(&self, block_height: u32) -> Result<Block, LedgerError> {
        let qr = match self.database.lock().unwrap().query_row(
            "SELECT raw_body FROM tmpblocks WHERE block_height = ?1",
            params![block_height],
            |row| Ok(row.get::<_, Vec<u8>>(0).unwrap()),
        ) {
            Ok(qr) => qr,
            Err(e) => {
                return Err(LedgerError::Block(format!(
                    "Couldn't find any block with block height {}: {}",
                    block_height, e
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

    pub fn create_block(&self, transactions: Vec<Transaction>) -> Result<Block, LedgerError> {
        let prev_block_height = self.get_block_height();
        let prev_block_time = self.get_block_time(prev_block_height).unwrap();

        let prev_blockhash = match self.get_block_with_height(prev_block_height) {
            Ok(b) => b.block_hash(),
            Err(_) => BlockHash::all_zeros(),
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

    fn calculate_merkle_root(&self, txids: Vec<Txid>) -> Result<TxMerkleNode, LedgerError> {
        let leaves: Vec<_> = txids
            .iter()
            .map(|txid| txid.to_raw_hash().as_byte_array().to_owned())
            .collect();

        let merkle_tree = MerkleTree::<Sha256>::from_leaves(leaves.as_slice());

        let root = match merkle_tree.root() {
            Some(r) => r,
            None => {
                return Err(LedgerError::Transaction(format!(
                    "Not enough transactions ({}) are given to create a merkle tree",
                    txids.len()
                )))
            }
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
    pub fn get_block_height(&self) -> u32 {
        self.database
            .lock()
            .unwrap()
            .query_row("SELECT height FROM block_height", params![], |row| {
                let body = row.get::<_, i64>(0).unwrap();

                Ok(body as u32)
            })
            .unwrap()
    }

    /// Returns specified transaction's block height.
    ///
    /// # Panics
    ///
    /// Will panic if cannot get height from database.
    pub fn get_tx_block_height(&self, txid: Txid) -> u32 {
        self.database
            .lock()
            .unwrap()
            .query_row(
                "SELECT (block_height) FROM transactions WHERE txid = ?1",
                params![txid.to_string()],
                |row| {
                    let body = row.get::<_, i64>(0).unwrap();

                    Ok(body as u32)
                },
            )
            .unwrap()
    }

    /// Sets block height to given value.
    ///
    /// # Panics
    ///
    /// Will panic if cannot set height to database.
    fn set_block_height(&self, height: u32) {
        self.database
            .lock()
            .unwrap()
            .execute("UPDATE block_height SET height = ?1", params![height])
            .unwrap();
    }

    /// Increments block height by 1 and sets block time of the next block 10
    /// minutes after the previous block time.
    ///
    /// # Panics
    ///
    /// Will panic if either [`get_block_height`] or [`set_block_height`]
    /// panics.
    pub fn increment_block_height(&self) {
        let last_block_height = self.get_block_height();
        let current_block_height = last_block_height + 1;

        let last_block_time = if last_block_height == 0 {
            // This is genesis block. Use current time.
            let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

            // Return 10 minutes before current time. New block will have the
            // time of 10 minute after the last block.
            (duration - Duration::from_secs(60 * 10)).as_secs() as u32
        } else {
            self.get_block_time(last_block_height).unwrap()
        };
        let current_block_time = last_block_time + (60 * 10);

        self.set_block_time(current_block_height, current_block_time);
        self.set_block_height(current_block_height);
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
        if let Ok(time) = self.database.lock().unwrap().query_row(
            "SELECT unix_time FROM blocks WHERE block_height = ?1",
            params![block_height],
            |row| {
                let body = row.get::<_, i64>(0).unwrap();

                Ok(body as u32)
            },
        ) {
            return Ok(time);
        };

        Err(LedgerError::BlockInMempool(block_height))
    }

    /// Sets specified blocks time.
    ///
    /// # Panics
    ///
    /// Will panic if there is a problem with database.
    fn set_block_time(&self, block_height: u32, time: u32) {
        self.database
            .lock()
            .unwrap()
            .execute(
                "INSERT INTO blocks (block_height, unix_time) VALUES (?1, ?2)",
                params![block_height, time],
            )
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::ledger::Ledger;
    use bitcoin::{Amount, ScriptBuf, Transaction, Txid};

    #[test]
    fn get_set_block_height() {
        let ledger = Ledger::new("get_set_block_height");

        let current_height = ledger.get_block_height();
        assert_eq!(current_height, 0);

        ledger.set_block_height(0x45);
        ledger.set_block_time(0x45, 0);
        let current_height = ledger.get_block_height();
        assert_eq!(current_height, 0x45);

        ledger.set_block_height(0x1F);
        ledger.set_block_time(0x1F, 0);
        let current_height = ledger.get_block_height();
        assert_eq!(current_height, 0x1F);
    }

    #[test]
    fn increment_block_height() {
        let ledger = Ledger::new("increment_block_height");

        let current_height = ledger.get_block_height();
        assert_eq!(current_height, 0);

        ledger.increment_block_height();
        let current_height = ledger.get_block_height();
        assert_eq!(current_height, 1);

        // Because we aren't mining blocks rn, we must add block times.
        ledger.set_block_time(0x44, 0);
        ledger.set_block_height(0x45);
        let current_height = ledger.get_block_height();
        assert_eq!(current_height, 0x45);

        // Because we aren't mining blocks rn, we must add block times.
        ledger.set_block_time(0x45, 0);
        ledger.increment_block_height();
        let current_height = ledger.get_block_height();
        assert_eq!(current_height, 0x46);
    }

    #[test]
    fn create_add_get_block() {
        let ledger = Ledger::new("create_add_get_block");
        ledger.increment_block_height();
        ledger.increment_block_height();
        let block_heigh = ledger.get_block_height();

        let mut transactions: Vec<Transaction> = Vec::new();
        for i in 0..0x45 {
            let txout = ledger.create_txout(Amount::from_sat(0x45 * i), ScriptBuf::new());
            let tx = ledger.create_transaction(vec![], vec![txout]);

            transactions.push(tx);
        }

        let block = ledger.create_block(transactions).unwrap();

        ledger.add_block(block.clone()).unwrap();

        let read_block = ledger.get_block_with_height(block_heigh).unwrap();

        assert_eq!(block, read_block);
    }

    #[test]
    fn merkle_tree() {
        let ledger = Ledger::new("merkle_tree");

        let txids = [
            Txid::from_str("39bd74af2177428de4cfb10dc82af0b04d7d51859a4c501470734bbdc8e8e633")
                .unwrap(),
            Txid::from_str("353f5e73fa737f625474b81a8d0a5ea00b23ce8ff8880cf001e3d472d325bc93")
                .unwrap(),
            Txid::from_str("353f5e73fa737f625474b81a8d0a5ea00b23ce8ff8880cf001e3d472d325bc93")
                .unwrap(),
            Txid::from_str("9c9a8f998468bb363e5809ce84a80e35054f104b64ef4aa2d832a426e6837665")
                .unwrap(),
        ];
        println!("Txids: {:?}", txids);

        let merkle_root = ledger.calculate_merkle_root(txids.to_vec().clone());

        println!("Merkle root: {:?}", (merkle_root));
    }
}
