## 🌟 Swift Network
<div align="center">

[English](./README.md) | [中文](./README_zh.md)

> 一个高性能的区块链平台，使用 Move 智能合约

### 📋 目录
概述
架构
核心模块
入门指南
文档

🚀 概述
Swift Network 是一个高性能的区块链平台，使用 Rust 构建，具有 Move 智能合约和面向对象的数据模型。它结合了现代区块链设计的最佳实践，并创新性地提供可扩展性和安全性功能。

✨ 主要特性
🔗 面向对象的数据模型
📜 Move 智能合约
🌐 高性能 P2P 网络
🔒 Narwhal-Bullshark 共识
📊 全面的监控

🏗 架构
```txt
graph TD
    A[客户端] --> B[网络层]
    B --> C[事务处理]
    C --> D[共识层]
    D --> E[执行引擎]
    E --> F[存储层]
```
💎 核心模块
1. 权威模块 🏛
验证节点管理与协调
```txt
authority/
├── authority.rs       # 验证节点核心逻辑
├── authority_store.rs # 验证状态存储
├── checkpoint_store.rs# 检查点管理
├── epoch_manager.rs   # 纪元管理
├── mod.rs            # 模块接口
└── validator.rs      # 验证器实现
```
2. 配置模块 ⚙️
系统配置管理
```txt
config/
├── genesis.rs        # 创世配置
└── mod.rs           # 配置管理
```
3. 共识模块 🔄
Narwhal-Bullshark 共识实现
```txt
consensus/
├── bullshark.rs     # BullShark 共识
├── dag.rs           # DAG 结构
├── mod.rs           # 模块接口
├── narwhal.rs       # Narwhal 共识
├── safety_rules.rs  # 安全规则
└── types.rs         # 共识类型
```
4. 核心模块 🎯
核心数据结构和类型
```txt
core/
├── mod.rs           # 模块接口
├── object.rs        # 对象模型
└── types.rs         # 核心类型
```
5. 加密模块 🔐
加密原语
```txt
crypto/
├── keypair.rs       # 密钥对管理
├── mod.rs           # 模块接口
└── signature.rs     # 数字签名
```
6. 执行模块 ⚡
事务执行引擎
```txt
execution/
├── effects.rs       # 执行效果
├── executor.rs      # 事务执行器
├── gas.rs           # 燃料管理
├── mod.rs           # 模块接口
└── validator.rs     # 执行验证
```
7. 框架模块 📚
Move 框架实现
```txt
framework/
├── abilities.rs     # 对象能力
├── contracts/       # 系统合约
└── mod.rs           # 模块接口
```
8. 索引模块 📇
区块链数据索引服务
```txt
indexer/
├── builder.rs       # 索引构建器
├── mod.rs           # 模块接口
├── reader.rs        # 索引读取器
└── store.rs         # 索引存储
```
9. 内存池模块 💾
事务内存池
```txt
mempool/
├── mod.rs           # 模块接口
├── pool.rs          # 事务池
└── prioritizer.rs   # 事务优先级
```
10. 监控模块 📊
系统监控与指标
```txt
metrics/
├── metrics.rs      # 指标实现
└── mod.rs          # 模块接口
```
11.  网络模块 🌐
P2P 网络
```txt
network/
├── mod.rs          # 模块接口
└── p2p.rs          # P2P 实现
```
12.  协议模块 📜
核心协议定义
```txt
protocol/
├── certificate.rs   # 交易证书
├── errors.rs       # 协议错误
├── messages.rs     # 协议消息
├── mod.rs          # 模块接口
└── types.rs        # 协议类型
```
13.  法定驱动模块 🚗
共识驱动实现
```txt
quorum_driver/
├── driver.rs       # 法定驱动
└── mod.rs          # 模块接口
```
14.  运行时模块 ⚡
Move 虚拟机运行时
```txt
runtime/
├── execution/      # 执行上下文
├── mod.rs          # 模块接口
└── move_vm.rs      # Move 虚拟机实现
```
15. 状态模块 📦
状态管理
```txt
state/
├── accumulator.rs  # 状态累加器
├── checkpoint.rs   # 状态检查点
├── mod.rs          # 模块接口
├── pruner.rs       # 状态修剪
└── store.rs        # 状态存储
```
16.  存储模块 💽
持久存储
```txt
storage/
├── cache.rs        # 存储缓存
├── event_store.rs  # 事件存储
├── indexes.rs      # 存储索引
├── mod.rs          # 模块接口
├── object_store.rs  # 对象存储
└── rocks_store.rs  # RocksDB 实现
```
17. Swift Network 系统模块 🎮
系统合约和治理
```txt
swift_system/
├── genesis.rs      # 创世配置
├── governance.rs   # 治理系统
├── mod.rs          # 模块接口
├── rewards.rs      # 奖励系统
├── stake.rs        # 质押系统
└── validators.rs    # 验证器管理
```
18.  遥测模块 📡
系统监控和日志记录
```txt
telemetry/
├── logging.rs      # 日志系统
├── metrics.rs      # 指标收集
├── mod.rs          # 模块接口
└── tracing.rs      # 分布式追踪
```
19.  事务模块 💳
事务处理
```txt
transaction/
├── manager.rs      # 事务管理
├── mod.rs          # 模块接口
└── validator.rs    # 事务验证
```
20.  工具模块 🛠
工具函数
```txt
utils/
├── crypto.rs       # 加密工具
└── mod.rs          # 模块接口
```
🚀 入门指南
前提条件
Rust 1.70+
Cargo
RocksDB

安装
```bash
# 克隆仓库
git clone https://github.com/your-username/swift-network.git

# 构建项目
cargo build --release

# 运行测试
cargo test
```
📖 文档
API 文档
生成并查看 API 文档：
```bash
cargo doc --open
```
配置
示例配置文件：
```toml
[network]
listen_address = "0.0.0.0:8080"
max_peers = 50

[consensus]
consensus_type = "BullShark"
block_time_ms = 2000

[storage]
db_path = "data/swift-network"
```
🤝 贡献
我们欢迎贡献！请参阅我们的贡献指南以获取详细信息。

📄 许可
本项目根据 MIT 许可证授权。

🔗 链接

项目网站
文档
GitHub 仓库
---
由 Swift Network 团队❤️构建。