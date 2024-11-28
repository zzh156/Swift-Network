use hyper::{Body, Request, Response, Server, Method};
use hyper::service::{make_service_fn, service_fn};
use serde_json::json;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use crate::blockchain::Dag;
use crate::transaction::Transaction;

async fn handle_request(req: Request<Body>, dag: Arc<Mutex<Dag>>) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        // 处理 POST 请求
        (&Method::POST, "/transaction") => {
            let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
            
            // 打印接收到的请求
            println!("收到请求: {}", body_str);
            
            match serde_json::from_str::<Transaction>(&body_str) {
                Ok(transaction) => {
                    let mut dag = dag.lock().unwrap();
                    if dag.validate_transaction(&transaction) {
                        dag.add_transaction(transaction.clone()).unwrap();
                        // 返回成功响应
                        let response = json!({ "status": "success", "message": "交易已添加到交易池" });
                        Ok(Response::new(Body::from(response.to_string())))
                    } else {
                        // 返回失败响应
                        let response = json!({ "status": "error", "message": "交易验证失败" });
                        Ok(Response::new(Body::from(response.to_string())))
                    }
                },
                Err(e) => {
                    let response = json!({ "status": "error", "message": format!("无效的交易格式: {}", e) });
                    Ok(Response::new(Body::from(response.to_string())))
                }
            }
        },
        _ => {
            // 处理其他请求或默认情况
            Ok(Response::new(Body::from("404 Not Found")))
        }
    }
}

pub async fn start_http_server(dag: Arc<Mutex<Dag>>, address: &str) {
    let make_svc = make_service_fn(move |_conn| {
        let dag_for_handler = Arc::clone(&dag); // 这里克隆 Arc
        async move { 
            Ok::<_, Infallible>(service_fn(move |req| handle_request(req, dag_for_handler.clone()))) 
        }
    });

    let server = Server::bind(&address.parse().unwrap())
        .serve(make_svc);

    println!("HTTP 服务正在监听: {}", address);
    if let Err(e) = server.await {
        eprintln!("服务器错误: {}", e);
    }
}