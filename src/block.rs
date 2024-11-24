use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::transaction::Transaction;
use crate::utils::{current_timestamp, calculate_merkle_root};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,                      // 区块高度
    pub timestamp: u64,                  // 区块生成时间戳
    pub transactions: Vec<Transaction>,  // 区块包含的交易
    pub merkle_root: String,             // 默克尔树根哈希
    pub previous_hash: String,           // 前一个区块的哈希
    pub hash: String,                    // 当前区块的哈希
    pub nonce: u64,                      // 随机数（用于工作量证明）
    pub mined_by: String,                // 挖出该区块的矿工地址
}

impl Block {
    pub fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String) -> Self {
        let merkle_root = calculate_merkle_root(&transactions);
        Block {
            index,
            timestamp: current_timestamp(),
            transactions,
            merkle_root,
            previous_hash,
            hash: String::new(),
            nonce: 0,
            mined_by: String::new(),
        }
    }

    // 计算区块的哈希值
    pub fn calculate_hash(&self) -> String {
        let block_content = format!(
            "{}{}{}{}{}{}",
            self.index, self.timestamp, self.merkle_root, self.transactions.len(), self.previous_hash, self.nonce
        );
        let mut hasher = Sha256::new();
        hasher.update(block_content);
        let result = hasher.finalize();
        hex::encode(result)
    }

    // 挖矿逻辑
    pub fn mine_block(&mut self, difficulty: usize, miner_address: &str) {
        self.mined_by = miner_address.to_string();
        let target = "0".repeat(difficulty); // 挖矿目标
        while !self.hash.starts_with(&target) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
        println!(
            "矿工 {} 成功挖出区块: {} (Nonce: {})",
            miner_address, self.hash, self.nonce
        );
    }
}
