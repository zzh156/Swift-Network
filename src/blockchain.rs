use crate::block::DagNode;
use crate::transaction::Transaction;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug)]
pub struct Dag {
    pub graph: HashMap<String, DagNode>,       // 存储 DAG 节点的图结构（哈希 -> 节点）
    pub transaction_pool: Vec<Transaction>,   // 待处理交易池
    pub accounts: HashMap<String, u64>,       // 账户余额
    pub confirmed_nodes: HashSet<String>,     // 已确认的节点哈希集合
}

impl Dag {
    pub fn new() -> Self {
        Dag {
            graph: HashMap::new(),
            transaction_pool: Vec::new(),
            accounts: HashMap::new(),
            confirmed_nodes: HashSet::new(),
        }
    }

    // 添加创世节点
    pub fn add_genesis_node(&mut self) {
        let genesis_node = DagNode::new(vec![], vec![]);
        self.graph.insert(genesis_node.hash.clone(), genesis_node);
    }

    // 验证并添加交易到交易池
    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<(), String> {
        // 打印交易内容，方便调试
        println!("添加交易: {:?}", transaction);
    
        if self.validate_transaction(&transaction) {
            self.transaction_pool.push(transaction);
            Ok(())
        } else {
            Err("交易失败：余额不足或签名无效！".to_string())
        }
    }
    

    // 验证交易
    pub fn validate_transaction(&self, transaction: &Transaction) -> bool {
        let sender_balance = self.accounts.get(&transaction.sender).cloned().unwrap_or(0);
        let valid_signature = transaction.verify_signature();
        
        println!("验证交易: 发送方余额 = {}, 交易金额 = {}, 签名有效 = {}", sender_balance, transaction.amount, valid_signature);
    
        sender_balance >= transaction.amount && valid_signature
    }
    

    // 创建新的 DAG 节点
    pub fn create_new_node(&mut self) {
        let parent_hashes: Vec<String> = self.graph.keys().cloned().collect();
        let transactions = self.transaction_pool.clone();
        let new_node = DagNode::new(transactions, parent_hashes);

        for tx in &new_node.transactions {
            if tx.sender != "系统奖励" {
                *self.accounts.entry(tx.sender.clone()).or_insert(0) -= tx.amount;
            }
            *self.accounts.entry(tx.receiver.clone()).or_insert(0) += tx.amount;
        }

        self.transaction_pool.clear();
        self.graph.insert(new_node.hash.clone(), new_node.clone());
        self.update_weights(&new_node);
    }

    // 更新 DAG 权重（确认机制）
    fn update_weights(&mut self, new_node: &DagNode) {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(new_node.hash.clone());

        while let Some(current_hash) = queue.pop_front() {
            if visited.contains(&current_hash) {
                continue;
            }
            visited.insert(current_hash.clone());

            if let Some(node) = self.graph.get_mut(&current_hash) {
                node.weight += 1;
                for parent_hash in &node.parent_hashes {
                    queue.push_back(parent_hash.clone());
                }
            }
        }
    }

    // 确认交易节点（基于权重）
    pub fn confirm_transactions(&mut self) {
        for node in self.graph.values() {
            if node.weight > 2 && !self.confirmed_nodes.contains(&node.hash) {
                self.confirmed_nodes.insert(node.hash.clone());
                println!("确认节点: {}", node.hash);
            }
        }
    }

    // 打印 DAG 状态
    pub fn print_status(&self) {
        println!("DAG 节点数量: {}", self.graph.len());
        println!("账户余额: {:?}", self.accounts);
        println!("已确认节点: {:?}", self.confirmed_nodes);
    }
}