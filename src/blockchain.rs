use crate::block::Block;
use crate::transaction::Transaction;
use crate::utils::{calculate_merkle_root};
use std::collections::{HashMap, HashSet};
use crate::utils::current_timestamp;

#[derive(Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,                  // 区块链
    pub transaction_pool: Vec<Transaction>, // 待处理交易池
    pub difficulty: usize,                  // 挖矿难度
    pub mining_reward: u64,                 // 挖矿奖励
    pub accounts: HashMap<String, u64>,     // 账户余额
    pub utxo_set: HashSet<String>,          // UTXO 集合，存储未花费的交易输出
}

impl Blockchain {
    pub fn new(difficulty: usize, mining_reward: u64) -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            transaction_pool: Vec::new(),
            difficulty,
            mining_reward,
            accounts: HashMap::new(),
            utxo_set: HashSet::new(),
        };
        blockchain.add_genesis_block();
        blockchain.accounts.insert("Alice".to_string(), 100);
        blockchain.accounts.insert("Bob".to_string(), 50);
        blockchain
    }
    pub fn add_genesis_block(&mut self) {
        let genesis_block = Block {
            index: 0,
            timestamp: current_timestamp(),
            merkle_root: "0".to_string(), // 示例值，之后可以计算
            transactions: vec![],
            previous_hash: "0".to_string(),
            hash: "0".repeat(self.difficulty),
            nonce: 0,
            mined_by: "系统".to_string(),
        };
        self.chain.push(genesis_block);
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        // 检查 UTXO 是否有效
        if !self.validate_transaction(&transaction) {
            println!("交易失败：无效的 UTXO 或余额不足！");
            return;
        }

        self.transaction_pool.push(transaction);
    }
    // 验证交易是否有效
    fn validate_transaction(&self, transaction: &Transaction) -> bool {
        let sender_balance = self.accounts.get(&transaction.sender).cloned().unwrap_or(0);
        sender_balance >= transaction.amount
    }

    pub fn integrate_new_block(&mut self, block: Block) {
        let expected_merkle_root = calculate_merkle_root(&block.transactions);

        if block.index == self.chain.len() as u64
            && block.previous_hash == self.chain.last().unwrap().hash
            && block.merkle_root == expected_merkle_root
        {
            self.chain.push(block.clone());

            // 更新 UTXO 和账户余额
            for tx in &block.transactions {
                if tx.sender != "系统奖励" {
                    let utxo_key = format!("{}:{}", tx.sender, tx.amount);
                    self.utxo_set.remove(&utxo_key);  // 移除发送方的 UTXO

                    *self.accounts.entry(tx.sender.clone()).or_insert(0) -= tx.amount;
                }

                // 添加接收方的 UTXO
                let utxo_key = format!("{}:{}", tx.receiver, tx.amount);
                self.utxo_set.insert(utxo_key);

                *self.accounts.entry(tx.receiver.clone()).or_insert(0) += tx.amount;
            }
        } else {
            println!("接收到无效区块！");
        }
    }
}
