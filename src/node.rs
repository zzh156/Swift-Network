use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use serde_json::json;
use crate::blockchain::Dag;
use crate::transaction::Transaction;

pub fn start_node(dag: Arc<Mutex<Dag>>, address: &str) {
    let listener = TcpListener::bind(address).expect("无法绑定到指定端口！");
    println!("节点启动，监听地址: {}", address);

    for stream in listener.incoming() {
        let stream = stream.expect("连接处理失败！");
        let dag_for_handler = Arc::clone(&dag);
        thread::spawn(move || {
            handle_connection(stream, dag_for_handler);
        });
    }
}

fn handle_connection(mut stream: TcpStream, dag: Arc<Mutex<Dag>>) {
    let mut buffer = [0; 1024];
    if stream.read(&mut buffer).is_err() {
        println!("接收请求时发生错误！");
        return;
    }

    let request: String = String::from_utf8_lossy(&buffer[..]).trim().to_string();
    println!("收到请求: {}", request);  // 打印接收到的交易请求

    match serde_json::from_str::<Transaction>(&request) {
        Ok(transaction) => {
            println!("解析的交易: {:?}", transaction);  // 打印解析后的交易内容

            let mut dag = dag.lock().unwrap();

            // 验证交易
            if !dag.validate_transaction(&transaction) {
                let response = json!({ "status": "error", "message": "交易验证失败" });
                stream.write_all(response.to_string().as_bytes()).unwrap();
                println!("交易验证失败: {:?}", transaction);
                return;
            }

            // 尝试将交易添加到交易池
            match dag.add_transaction(transaction.clone()) {
                Ok(_) => {
                    let response = json!({ "status": "success", "message": "交易已添加到交易池" });
                    stream.write_all(response.to_string().as_bytes()).unwrap();
                    println!("交易已添加到交易池");
                    
                    // 每次成功添加交易后打印区块链状态
                    dag.print_status();
                }
                Err(reason) => {
                    let response = json!({ "status": "error", "message": reason });
                    stream.write_all(response.to_string().as_bytes()).unwrap();
                    println!("交易失败: {}", reason);
                }
            }
        }
        Err(e) => {
            let response = json!({ "status": "error", "message": format!("无效的交易格式: {}", e) });
            stream.write_all(response.to_string().as_bytes()).unwrap();
            println!("无效的交易格式: {}", e); // 打印解析错误
        }
    }
}
