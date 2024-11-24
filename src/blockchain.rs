use crate::block::Block;
use crate::transaction::Transaction;
use crate::utils::current_timestamp;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,                // 区块链
    pub transaction_pool: Vec<Transaction>, // 待处理交易池
    pub difficulty: usize,                // 挖矿难度
    pub mining_reward: u64,               // 挖矿奖励
    pub accounts: HashMap<String, u64>,   // 账户余额
}

impl Blockchain {
    pub fn new(difficulty: usize, mining_reward: u64) -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            transaction_pool: Vec::new(),
            difficulty,
            mining_reward,
            accounts: HashMap::new(),
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
            transactions: vec![],
            previous_hash: "0".to_string(),
            hash: "0".repeat(self.difficulty),
            nonce: 0,
            mined_by: "系统".to_string(),
        };
        self.chain.push(genesis_block);
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        // 检查账户余额
        if let Some(sender_balance) = self.accounts.get(&transaction.sender) {
            if *sender_balance < transaction.amount {
                println!("交易失败：余额不足！");
                return;
            }
        } else {
            println!("交易失败：发送方账户不存在！");
            return;
        }
        self.transaction_pool.push(transaction);
    }

    pub fn integrate_new_block(&mut self, block: Block) {
        if block.index == self.chain.len() as u64
            && block.previous_hash == self.chain.last().unwrap().hash
        {
            self.chain.push(block.clone());
            for tx in &block.transactions {
                if tx.sender != "系统奖励" {
                    *self.accounts.entry(tx.sender.clone()).or_insert(0) -= tx.amount;
                }
                *self.accounts.entry(tx.receiver.clone()).or_insert(0) += tx.amount;
            }
        } else {
            println!("接收到无效块！");
        }
    }
}
