use std::io::{Read};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::blockchain::Blockchain;
use crate::block::Block;
use crate::transaction::Transaction;

pub fn start_node(blockchain: Arc<Mutex<Blockchain>>, address: String, peers: Vec<String>) {
    let listener = TcpListener::bind(&address).unwrap();
    println!("节点启动，监听地址: {}", address);

    // 克隆 Arc<Mutex<Blockchain>>，以便线程内使用
    let blockchain_for_miner = Arc::clone(&blockchain);
    let miner_address = address.clone();
    
    // 挖矿线程
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(100));
            let new_block;
            {
                let blockchain = blockchain_for_miner.lock().unwrap();
                if blockchain.transaction_pool.is_empty() {
                    continue;
                }

                // 创建包含奖励交易的交易池
                let mut transactions = blockchain.transaction_pool.clone();
                let reward_transaction = Transaction::new("系统奖励".to_string(), miner_address.clone(), blockchain.mining_reward);
                transactions.push(reward_transaction);

                new_block = Block::new(
                    blockchain.chain.len() as u64,
                    transactions,
                    blockchain.chain.last().unwrap().hash.clone(),
                );
            }

            // 使 new_block 可变，并执行挖矿
            let mut new_block = new_block;
            new_block.mine_block(blockchain_for_miner.lock().unwrap().difficulty, &miner_address);

            // 区块挖掘成功后将其加入区块链
            let mut blockchain = blockchain_for_miner.lock().unwrap();
            blockchain.integrate_new_block(new_block);
        }
    });

    // 接受传入连接
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let blockchain_for_handler = Arc::clone(&blockchain); // 在主线程中克隆 Arc
        let peers = peers.clone();
        thread::spawn(move || {
            handle_connection(stream, blockchain_for_handler, peers);
        });
    }
}


fn handle_connection(mut stream: TcpStream, blockchain: Arc<Mutex<Blockchain>>, _peers: Vec<String>) {
    let mut buffer = [0; 512];
    if stream.read(&mut buffer).is_err() {
        println!("接收消息时发生错误！");
        return;
    }
    let request: String = String::from_utf8_lossy(&buffer[..]).trim().to_string();
    if let Ok(block) = serde_json::from_str::<Block>(&request) {
        let mut blockchain = blockchain.lock().unwrap();
        blockchain.integrate_new_block(block);
    }
}
