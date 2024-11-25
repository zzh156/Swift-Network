use std::io::{Read};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::blockchain::Dag;
use crate::block::DagNode;

pub fn start_node(dag: Arc<Mutex<Dag>>, address: String) {
    let listener = TcpListener::bind(&address).unwrap();
    println!("节点启动，监听地址: {}", address);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let dag_for_handler = Arc::clone(&dag);
        thread::spawn(move || {
            handle_connection(stream, dag_for_handler);
        });
    }
}

fn handle_connection(mut stream: TcpStream, dag: Arc<Mutex<Dag>>) {
    let mut buffer = [0; 512];
    if stream.read(&mut buffer).is_err() {
        println!("接收消息时发生错误！");
        return;
    }
    let request: String = String::from_utf8_lossy(&buffer[..]).trim().to_string();
    if let Ok(node) = serde_json::from_str::<DagNode>(&request) {
        let mut dag = dag.lock().unwrap();
        dag.graph.insert(node.hash.clone(), node);
    }
}