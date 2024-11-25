mod blockchain;
mod block;
mod transaction;
mod utils;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use blockchain::Dag;
use transaction::Transaction;

fn main() {
    // 创建 DAG 实例
    let dag = Arc::new(Mutex::new(Dag::new()));

    // 初始化 DAG，添加创世节点
    {
        let mut dag = dag.lock().unwrap();
        dag.add_genesis_node();
        dag.accounts.insert("Alice".to_string(), 100);
        dag.accounts.insert("Bob".to_string(), 50);
    }

    // 定期打印 DAG 状态
    let dag_for_output = Arc::clone(&dag);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(5));
            let dag = dag_for_output.lock().unwrap();
            println!("\n--- DAG 状态 ---");
            dag.print_status();
        }
    });

    // 初始化交易
    {
        let mut dag = dag.lock().unwrap();
        dag.add_transaction(Transaction::new("Alice".to_string(), "Bob".to_string(), 10));
    }

    // 定期创建新节点
    let dag_for_node_creation = Arc::clone(&dag);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(10));
            let mut dag = dag_for_node_creation.lock().unwrap();
            dag.create_new_node();
        }
    });

    // 定期确认交易
    let dag_for_confirmation = Arc::clone(&dag);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(15));
            let mut dag = dag_for_confirmation.lock().unwrap();
            dag.confirm_transactions();
        }
    });

    // 主线程保持运行
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}