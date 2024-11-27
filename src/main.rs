mod blockchain;
mod block;
mod crypto;
mod node;
mod transaction;
mod utils;

use blockchain::Dag;
use node::start_node;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use transaction::Transaction;
use ed25519_dalek::Keypair;  // 只导入 Keypair
use rand::rngs::OsRng;  // 导入 OsRng

fn main() {
    let dag = Arc::new(Mutex::new(Dag::new()));

    // 创建一个随机的密钥对
    let mut rng = OsRng;  // 直接使用 OsRng
    let keypair = Keypair::generate(&mut rng);  // 使用 `generate` 方法

    {
        let mut dag = dag.lock().unwrap();
        dag.add_genesis_node(); // 初始化创世节点
        dag.accounts.insert("Alice".to_string(), 100); // 初始化账户 Alice
        dag.accounts.insert("Bob".to_string(), 50);   // 初始化账户 Bob

        // 创建并添加默认交易
        let mut default_transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            10, // 交易金额
            1,  // 交易费用
        );

        // 为交易签名
        default_transaction.sign(&keypair); // 填充 signature 和 public_key

        // 添加交易到 DAG
        dag.add_transaction(default_transaction);
    }

    // 启动节点监听 HTTP 请求
    let dag_for_node = Arc::clone(&dag);
    thread::spawn(move || {
        start_node(dag_for_node, "127.0.0.1:8080");
    });

    // 每 5 秒生成一个新的区块并更新 DAG
    let dag_for_block_generation = Arc::clone(&dag);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(5)); // 每 5 秒生成一个新区块
            let mut dag = dag_for_block_generation.lock().unwrap();

            // 创建新的 DAG 节点并将交易加入图中
            dag.create_new_node();

            // 确认基于 DAG 权重的交易节点
            dag.confirm_transactions();

            // 打印当前区块链状态
            dag.print_status();
        }
    });

    // 保持主线程运行
    loop {
        std::thread::park(); // 阻塞主线程，保持程序运行
    }
}
