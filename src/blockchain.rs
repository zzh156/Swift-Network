use crate::block::DagNode;
use crate::transaction::Transaction;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug)]
pub struct Dag {
    pub graph: HashMap<String, DagNode>, // 存储 DAG 节点的图结构（哈希 -> 节点）
    pub transaction_pool: Vec<Transaction>, // 待处理交易池
    pub accounts: HashMap<String, u64>, // 账户余额
    pub confirmed_nodes: HashSet<String>, // 已确认的节点哈希集合
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
        let genesis_node = DagNode::new(vec![], vec![]); // 创世节点没有父节点和交易
        self.graph.insert(genesis_node.hash.clone(), genesis_node);
    }

    // 验证并添加交易到交易池
    pub fn add_transaction(&mut self, transaction: Transaction) {
        if self.validate_transaction(&transaction) {
            self.transaction_pool.push(transaction);
        } else {
            println!("交易失败：余额不足或无效交易！");
        }
    }

    // 验证交易
    fn validate_transaction(&self, transaction: &Transaction) -> bool {
        let sender_balance = self.accounts.get(&transaction.sender).cloned().unwrap_or(0);
        sender_balance >= transaction.amount
    }

    // 创建新的 DAG 节点
    pub fn create_new_node(&mut self) {
        let parent_hashes: Vec<String> = self.graph.keys().cloned().collect(); // 选择所有现有节点作为父节点
        let transactions = self.transaction_pool.clone();
        let new_node = DagNode::new(transactions, parent_hashes);

        // 更新账户余额
        for tx in &new_node.transactions {
            if tx.sender != "系统奖励" {
                *self.accounts.entry(tx.sender.clone()).or_insert(0) -= tx.amount;
            }
            *self.accounts.entry(tx.receiver.clone()).or_insert(0) += tx.amount;
        }

        // 清空交易池并添加新节点
        self.transaction_pool.clear();
        

        // 更新节点权重
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
                node.weight += 1; // 累积权重
                for parent_hash in &node.parent_hashes {
                    queue.push_back(parent_hash.clone());
                }
            }
        }
    }

    // 拓扑排序（用于共识机制）
    pub fn topological_sort(&self) -> Vec<DagNode> {
        let mut in_degree = HashMap::new();
        let mut zero_in_degree = VecDeque::new();
        let mut sorted_nodes = Vec::new();

        // 计算入度
        for (hash, node) in &self.graph {
            in_degree.insert(hash.clone(), 0);
        }
        for node in self.graph.values() {
            for parent_hash in &node.parent_hashes {
                *in_degree.get_mut(parent_hash).unwrap() += 1;
            }
        }

        // 找到所有入度为 0 的节点
        for (hash, degree) in &in_degree {
            if *degree == 0 {
                zero_in_degree.push_back(hash.clone());
            }
        }

        // Kahn 算法进行拓扑排序
        while let Some(hash) = zero_in_degree.pop_front() {
            if let Some(node) = self.graph.get(&hash) {
                sorted_nodes.push(node.clone());
                for parent_hash in &node.parent_hashes {
                    if let Some(degree) = in_degree.get_mut(parent_hash) {
                        *degree -= 1;
                        if *degree == 0 {
                            zero_in_degree.push_back(parent_hash.clone());
                        }
                    }
                }
            }
        }

        sorted_nodes
    }

    // 确认交易节点（基于权重）
    pub fn confirm_transactions(&mut self) {
        for node in self.graph.values() {
            if node.weight > 2 && !self.confirmed_nodes.contains(&node.hash) { // 示例权重阈值为 2
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