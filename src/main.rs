mod blockchain;
mod block;
mod transaction;
mod node;
mod utils;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use blockchain::Blockchain;
use transaction::Transaction;

fn main() {
    // 创建区块链实例，并使用线程安全的 Arc 和 Mutex 包装
    let blockchain = Arc::new(Mutex::new(Blockchain::new(5, 50))); // 挖矿难度: 5，奖励: 50
    let node_addresses = Arc::new(vec![
        "127.0.0.1:7878".to_string(),
        "127.0.0.1:7879".to_string(),
        "127.0.0.1:7880".to_string(),
    ]);

    // 启动每个节点
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
            node::start_node(blockchain_for_node, address, peers);
        });
    }

    // 定期打印区块链状态
    let blockchain_for_output = Arc::clone(&blockchain);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(5));
            let blockchain = blockchain_for_output.lock().unwrap();
            println!("\n--- 区块链状态 ---");
            println!("区块数量: {}", blockchain.chain.len());
            println!("账户余额: {:?}", blockchain.accounts);
            println!("\n");
        }
    });

    // 初始化交易
    {
        let mut blockchain = blockchain.lock().unwrap();
        blockchain.add_transaction(Transaction::new("Alice".to_string(), "Bob".to_string(), 10));
    }

    // 主线程保持运行
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
