# 🚀 基于 Rust 的有向无环图 (DAG) 区块链项目

这是一个使用 Rust 构建的 **有向无环图 (DAG)** 区块链项目，旨在帮助理解区块链的基础概念，例如交易处理、节点生成和简单的共识机制。本项目主要用于学习和探索，也为未来的功能扩展（例如 P2P 网络和智能合约）打下基础。

---

## ✨ 项目特点

- 🌟 **有向无环图 (DAG)**：通过节点间的关系和交易，支持更高效的交易处理。
- 💸 **交易处理**：支持用户间的交易，节点可以记录和验证交易。
- 🏦 **账户管理**：简单的账户系统，映射地址与余额。
- 🏗 **DAG 管理**：支持创世节点生成、新节点创建及链条更新。
- 🌐 **节点通信**：支持多节点间通过 TCP 进行通信，模拟去中心化。
- 🛠 **可扩展性**：未来将支持 **P2P 网络** 和 **智能合约**。

---

## 🗂 项目结构

```plaintext
├── block.rs          // DAG 节点结构体，包含交易和父节点信息
├── blockchain.rs     // DAG 核心，管理交易池、DAG 和账户余额
├── transaction.rs     // 交易结构体，记录发送方、接收方和交易金额
├── utils.rs          // 工具函数，获取时间戳和计算默克尔根
├── main.rs           // 主程序入口，启动节点并运行 DAG 逻辑
🎯 功能详解
💼 交易 (Transaction)
一个交易包含以下信息：

发送方地址 (sender)
接收方地址 (receiver)
转账金额 (amount)
🌟 示例代码：
```
```rust
let transaction = Transaction::new("Alice".to_string(), "Bob".to_string(), 10);
dag.add_transaction(transaction);
```
```plaintext
⛓ DAG 节点 (DagNode)
DAG 节点是区块链的基本单元，包含以下内容：

节点 ID (id)
时间戳 (timestamp)
交易列表 (transactions)
父节点哈希 (parent_hashes)
默克尔树根哈希 (merkle_root)
当前节点的哈希 (hash)
节点的权重 (weight)
节点的哈希通过以下函数计算：
```

```rust

fn calculate_hash(&self) -> String {
    let node_content = format!(
        "{}{}{}{:?}",
        self.timestamp, self.merkle_root, self.parent_hashes.join(","), self.transactions
    );
    let mut hasher = Sha256::new();
    hasher.update(node_content);
    let result = hasher.finalize();
    hex::encode(result)
}
```
```plaintext
🪙 创建新节点
DAG 节点通过聚合交易并更新账户余额来创建：
```
```rust
pub fn create_new_node(&mut self) {
    let parent_hashes: Vec<String> = self.graph.keys().cloned().collect();
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
    self.graph.insert(new_node.hash.clone(), new_node);
}
```
```plaintext
📡 节点通信 (Networking)
每个节点通过 TCP 进行通信，能够接收新区块并将其广播给其他节点。

🚀 快速开始
1️⃣ 克隆项目代码
```
```bash
git clone https://github.com/zzh156/chain.git
cd chain
```
```plaintext
2️⃣ 构建项目
```
```bash
cargo build
```
3️⃣ 运行项目

```bash
cargo run
```
运行后，将会启动本地节点并模拟区块链操作。

🔮 未来功能
🌍 P2P 网络：支持真正的去中心化网络。
🧠 智能合约：添加基本的智能合约功能。
⚡️ 性能优化：提升区块链性能和可扩展性。
🔒 安全增强：通过加密算法提升数据安全性。
📜 项目许可证
本项目基于 MIT 协议，欢迎学习和使用！