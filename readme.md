# 🚀 基于 Rust 的工作量证明区块链项目

这是一个使用 Rust 构建的 **工作量证明 (Proof of Work)** 区块链项目，旨在帮助理解区块链的基础概念，例如交易处理、区块挖矿和简单的共识机制。本项目主要用于学习和探索，也为未来的功能扩展（例如 P2P 网络和智能合约）打下基础。

---

## ✨ 项目特点

- ⛏ **工作量证明 (PoW)**：通过计算解决挖矿难题来验证新区块。
- 💸 **交易处理**：支持用户间的交易，矿工可以打包并验证交易。
- 🏦 **账户管理**：简单的账户系统，映射地址与余额。
- 🏗 **区块链管理**：支持创世区块生成、新区块挖矿及链条更新。
- 🌐 **节点通信**：支持多节点间通过 TCP 进行通信，模拟去中心化。
- 🛠 **可扩展性**：未来将支持 **P2P 网络** 和 **智能合约**。

---

## 🗂 项目结构

```plaintext
├── Transaction       // 交易结构体，记录发送方、接收方、交易金额
├── Block             // 区块结构体，包括索引、交易列表、哈希等
├── Blockchain        // 区块链核心，管理交易池、区块链和账户余额
├── Networking        // 节点通信逻辑，支持广播区块和接收区块
└── main.rs           // 主程序入口，启动节点并运行区块链逻辑
🎯 功能详解
💼 交易 (Transaction)
一个交易包含以下信息：

发送方地址 (sender)
接收方地址 (receiver)
转账金额 (amount)
```
🌟 示例代码：

```rust
let transaction = Transaction::new("Alice".to_string(), "Bob".to_string(), 10);
blockchain.add_transaction(transaction);
```
```plaintext
⛓ 区块 (Block)
区块是区块链的基本单元，包含以下内容：

区块号 (index)
时间戳 (timestamp)
交易列表 (transactions)
上一区块的哈希值 (previous_hash)
当前区块的哈希值 (hash)
挖矿变量 (nonce)
矿工地址 (mined_by)
区块的哈希通过以下函数计算：
```

```rust
fn calculate_hash(&self) -> String {
    let block_content = format!(
        "{}{}{:?}{}{}",
        self.index, self.timestamp, self.transactions, self.previous_hash, self.nonce
    );
    let mut hasher = Sha256::new();
    hasher.update(block_content);
    hex::encode(hasher.finalize())
}
```
```plaintext
🪙 挖矿 (Mining)
矿工通过计算找到符合难度要求的哈希值来完成挖矿，并获得奖励。
```
```rust
fn mine_block(&mut self, difficulty: usize, miner_address: &str) {
    self.mined_by = miner_address.to_string();
    let target = "0".repeat(difficulty);
    while !self.hash.starts_with(&target) {
        self.nonce += 1;
        self.hash = self.calculate_hash();
    }
    println!("区块已挖出: {} (Nonce: {})", self.hash, self.nonce);
}
blockchain.mine_pending_transactions("Miner1".to_string());
```
```plaintext
📡 节点通信 (Networking)
每个节点都通过 TCP 通信，可以接收新区块并将其广播给其他节点。

🎨 可视化效果
下面是区块链的结构示意图，帮助理解区块的组成和链条的关系：
```
```plaintext
🚀 快速开始
1️⃣ 克隆项目代码
```bash
git clone https://github.com/zzh156/chain.git
cd chain
2️⃣ 构建项目
bash
复制代码
cargo build
3️⃣ 运行项目
bash
复制代码
cargo run
运行后，将会启动本地节点并模拟区块链操作。
```

🔮 未来功能
🌍 P2P 网络：支持真正的去中心化网络。
🧠 智能合约：添加基本的智能合约功能。
⚡️ 性能优化：提升区块链性能和可扩展性。
🔒 安全增强：通过加密算法提升数据安全性。
📜 项目许可证
本项目基于 MIT 协议，欢迎学习和使用！
```