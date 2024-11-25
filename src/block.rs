use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::transaction::Transaction;
use crate::utils::{current_timestamp, calculate_merkle_root};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DagNode {
    pub id: String, // DAG 节点的唯一标识
    pub timestamp: u64, // 节点生成时间戳
    pub transactions: Vec<Transaction>, // 节点包含的交易
    pub parent_hashes: Vec<String>, // 父节点哈希
    pub merkle_root: String, // 默克尔树根哈希
    pub hash: String, // 当前节点的哈希
    pub weight: u64, // 节点的权重（用于确认机制）
}

impl DagNode {
    pub fn new(transactions: Vec<Transaction>, parent_hashes: Vec<String>) -> Self {
        //计算交易集合的默克尔根，用于验证交易完整性。
        let merkle_root = calculate_merkle_root(&transactions);
        let mut node = DagNode {
            id: uuid::Uuid::new_v4().to_string(), // 使用 UUID 生成唯一标识
            timestamp: current_timestamp(),
            transactions,
            parent_hashes,
            merkle_root,
            hash: String::new(),
            weight: 0, // 初始权重为 0
        };
        node.hash = node.calculate_hash(); // 根据节点内容（时间戳、交易、父节点等）计算节点的哈希值，确保节点不可篡改。
        node
    }

    // 计算 DAG 节点的哈希值
    pub fn calculate_hash(&self) -> String {
        let node_content = format!(
            "{}{}{}{:?}",
            self.timestamp, self.merkle_root, self.parent_hashes.join(","), self.transactions
        );
        let mut hasher = Sha256::new();
        hasher.update(node_content);
        let result = hasher.finalize();
        hex::encode(result)
    }
}