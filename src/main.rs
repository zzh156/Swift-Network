use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// 交易结构体(发送者、接收者、数量)
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Transaction {
    sender: String,
    receiver: String,
    amount: u64,
}

impl Transaction {
    fn new(sender: String, receiver: String, amount: u64) -> Self {
        Transaction {
            sender,
            receiver,
            amount,
        }
    }
}

/// 区块结构
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Block {
    index: u64, //区块号
    timestamp: u64, //时间戳
    transactions: Vec<Transaction>, //区块中包含的所有交易
    previous_hash: String,//上一个区块的hash
    hash: String, //这个区块的hash
    nonce: u64,  //解决工作量证明难题的变量
    mined_by: String, // 添加字段，记录矿工地址
}

impl Block {
    fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String) -> Self {
        Block {
            index,
            timestamp: current_timestamp(),
            transactions,
            previous_hash,
            hash: String::new(),
            nonce: 0,
            mined_by: String::new(), // 初始化为空字符串
        }
    }

    fn calculate_hash(&self) -> String {
        let block_content = format!(
            "{}{}{:?}{}{}",
            self.index, self.timestamp, self.transactions, self.previous_hash, self.nonce
        );
        let mut hasher = Sha256::new();
        hasher.update(block_content);
        let result = hasher.finalize();
        hex::encode(result)
    }

    fn mine_block(&mut self, difficulty: usize, miner_address: &str) {
        self.mined_by = miner_address.to_string(); // 设置矿工地址
        let target = "0".repeat(difficulty);
        while !self.hash.starts_with(&target) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
        println!("区块已挖出: {} (Nonce: {})", self.hash, self.nonce);
    }
}

/// 区块链
#[derive(Debug)]
struct Blockchain {
    chain: Vec<Block>,//很多区块组成区块链
    transaction_pool: Vec<Transaction>,//交易内存池
    difficulty: usize,//难度
    mining_reward: u64,//挖矿奖励
    accounts: HashMap<String, u64>,//每个地址对应的余额
}

impl Blockchain {
    fn new(difficulty: usize, mining_reward: u64) -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            transaction_pool: Vec::new(),
            difficulty,
            mining_reward,
            accounts: HashMap::new(),
        };
        blockchain.add_genesis_block();
        //默认初始有两个钱包，分别给不同的余额
        blockchain.accounts.insert("Alice".to_string(), 100);
        blockchain.accounts.insert("Bob".to_string(), 50);

        blockchain
    }
    //添加创世区块
    fn add_genesis_block(&mut self) {
        //创世区块的序号是0，里面不包含交易，前一个区块hash是0
        let mut genesis_block = Block::new(0, vec![], "0".to_string());
        genesis_block.mine_block(self.difficulty, "系统奖励");
        self.chain.push(genesis_block);
    }
    //向交易内存池添加交易
    fn add_transaction(&mut self, transaction: Transaction) {
        //从区块链账户映射找到交易发起者的余额，或者没有这个发起者账户
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
    //挖矿打包内存池中的交易
    fn mine_pending_transactions(&mut self, miner_address: String) {
        //创建了一笔奖励交易，从“系统奖励”发送给矿工地址，用于奖励矿工打包区块
        let reward_transaction = Transaction::new(
            "系统奖励".to_string(),
            miner_address.clone(),
            self.mining_reward,
        );
        //奖励矿工交易放进内存池中，意味着奖励交易即将包含在被挖出的区块中
        self.transaction_pool.push(reward_transaction);

        let previous_hash = self.chain.last().unwrap().hash.clone();
        let mut new_block = Block::new(
            self.chain.len() as u64,
            self.transaction_pool.clone(),
            previous_hash,
        );

        new_block.mine_block(self.difficulty, &miner_address); // 传递矿工地址
        self.chain.push(new_block);
        self.transaction_pool.clear();

        let transactions = self.chain.last().unwrap().transactions.clone();
        for transaction in &transactions {
            self.update_balance(transaction);
        }
    }

    fn update_balance(&mut self, transaction: &Transaction) {
        if transaction.sender != "系统奖励" {
            *self.accounts.entry(transaction.sender.clone()).or_insert(0) -= transaction.amount;
        }
        *self.accounts.entry(transaction.receiver.clone()).or_insert(0) += transaction.amount;
    }
}

/// 获取当前时间戳（秒）
fn current_timestamp() -> u64 {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).expect("时间错误");
    since_epoch.as_secs()
}

/// 启动节点
fn start_node(blockchain: Arc<Mutex<Blockchain>>, address: String, peers: Vec<String>) {
    let listener = TcpListener::bind(&address).unwrap();
    println!("节点启动，监听地址: {}", address);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let blockchain = Arc::clone(&blockchain);
        let peers = peers.clone();
        thread::spawn(move || {
            handle_connection(stream, blockchain, peers);
        });
    }
}

/// 处理节点之间的连接
fn handle_connection(mut stream: TcpStream, blockchain: Arc<Mutex<Blockchain>>, peers: Vec<String>) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();
    let request: String = String::from_utf8_lossy(&buffer[..]).trim().to_string();

    if let Ok(block) = serde_json::from_str::<Block>(&request) {
        let mut blockchain = blockchain.lock().unwrap();

        if block.index == blockchain.chain.len() as u64
            && block.previous_hash == blockchain.chain.last().unwrap().hash
        {
            println!("接收到新块: {:?}", block.clone());
            blockchain.chain.push(block.clone());

            // 广播给其他节点
            for peer in &peers {
                let mut peer_stream = TcpStream::connect(peer).unwrap();
                let block_data = serde_json::to_string(&block).unwrap();
                peer_stream.write_all(block_data.as_bytes()).unwrap();
            }
        } else {
            println!("接收到无效块！");
        }
    }
}

/// 打印区块链状态
fn print_blockchain(blockchain: &Blockchain) {
    println!("当前区块链状态:");
    for block in &blockchain.chain {
        println!(
            "区块 #{} 被 {} 挖出，哈希值: {}",
            block.index, block.mined_by, block.hash
        );
    }
}

fn main() {
    let blockchain = Arc::new(Mutex::new(Blockchain::new(4, 50)));

    let node_addresses = Arc::new(vec![
        "127.0.0.1:7878".to_string(),
        "127.0.0.1:7879".to_string(),
        "127.0.0.1:7880".to_string(),
    ]);

    for address in node_addresses.iter() {
        let blockchain_for_node = Arc::clone(&blockchain);
        let node_addresses = Arc::clone(&node_addresses);
        let address = address.clone();

        thread::spawn(move || {
            let peers: Vec<String> = node_addresses
                .iter()
                .filter(|peer| **peer != address)
                .cloned()
                .collect();

            start_node(blockchain_for_node, address, peers);
        });
    }

    thread::sleep(std::time::Duration::from_secs(2)); // 确保节点启动完成

    {
        let mut blockchain = blockchain.lock().unwrap();
        blockchain.add_transaction(Transaction::new("Alice".to_string(), "Bob".to_string(), 10));
    }

    thread::sleep(std::time::Duration::from_secs(2)); // 模拟挖矿竞争

    {
        let mut blockchain = blockchain.lock().unwrap();
        blockchain.mine_pending_transactions("Miner1".to_string());
        print_blockchain(&blockchain); // 打印区块链状态
    }
}
