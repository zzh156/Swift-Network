mod blockchain;
mod block;
mod crypto;
mod node;
mod transaction;
mod utils;

use blockchain::Dag;
use node::start_http_server;  // 导入新的 HTTP 服务器启动函数
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use transaction::Transaction;
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;

fn main() {
    let dag = Arc::new(Mutex::new(Dag::new()));

    // 创建一个随机的密钥对
    let mut rng = OsRng;  
    let keypair = Keypair::generate(&mut rng); 

    {
        let mut dag = dag.lock().unwrap();
        dag.add_genesis_node(); 
        dag.accounts.insert("Alice".to_string(), 100);
        dag.accounts.insert("Bob".to_string(), 50);

        // 创建并添加默认交易
        let mut default_transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            10,
            1,
        );

        // 为交易签名
        default_transaction.sign(&keypair);

        // 添加交易到 DAG
        dag.add_transaction(default_transaction);
    }

    // 启动 HTTP 服务器监听交易请求
    let dag_for_node = Arc::clone(&dag);
    thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(start_http_server(dag_for_node, "127.0.0.1:8080"));
    });

    // 每 5 秒生成一个新的区块并更新 DAG
    let dag_for_block_generation = Arc::clone(&dag);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(5));
            let mut dag = dag_for_block_generation.lock().unwrap();
            dag.create_new_node();
            dag.confirm_transactions();
            dag.print_status();
        }
    });

    loop {
        std::thread::park(); 
    }
}
