use serde::{Deserialize, Serialize};
use crate::crypto::Crypto;
use ed25519_dalek::{Keypair, PublicKey, Signature};
use sha2::{Sha256, Digest}; // 引入 SHA-256 哈希库
use std::time::{SystemTime, UNIX_EPOCH}; // 用于获取当前时间
use rand::Rng; // 引入随机数生成库

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub sender: String,          // 交易发送方
    pub receiver: String,        // 交易接收方
    pub amount: u64,             // 交易金额
    pub fee: u64,                // 交易费用
    pub signature: Option<String>, // 签名
    pub public_key: Option<String>, // 公钥
    pub hash: String,            // 交易哈希值
}

impl Transaction {
    pub fn new(sender: String, receiver: String, amount: u64, fee: u64) -> Self {
        let mut transaction = Transaction {
            sender,
            receiver,
            amount,
            fee,
            signature: None,
            public_key: None,
            hash: String::new(), // 初始化时哈希为空
        };
        transaction.calculate_hash(); // 计算哈希
        transaction // 返回交易对象
    }

    // 计算交易的哈希值
    fn calculate_hash(&mut self) {
        // 获取当前时间戳
        let start = SystemTime::now();
        let timestamp = start.duration_since(UNIX_EPOCH).expect("时间戳获取失败").as_secs();

        // 生成随机数
        let nonce: u64 = rand::thread_rng().gen();

        // 包含发送者、接收者、金额、费用、时间戳和随机数
        let message = format!("{}{}{}{}{}{}", self.sender, self.receiver, self.amount, self.fee, timestamp, nonce);
        let mut hasher = Sha256::new();
        hasher.update(message);
        self.hash = format!("{:x}", hasher.finalize()); // 将哈希值存储为十六进制字符串
    }

    // 为交易签名
    pub fn sign(&mut self, keypair: &Keypair) {
        let signature = Crypto::sign_message(&self.hash, keypair);  // 使用哈希值签名
        self.signature = Some(hex::encode(signature.to_bytes()));  // 将签名转为十六进制字符串
        self.public_key = Some(hex::encode(keypair.public.to_bytes())); // 将公钥转为十六进制字符串
    }

    // 验证交易签名
    pub fn verify_signature(&self) -> bool {
        if let (Some(sig_hex), Some(pub_key_hex)) = (&self.signature, &self.public_key) {
            let sig_bytes = hex::decode(sig_hex).expect("签名解码失败");
            let pub_key_bytes = hex::decode(pub_key_hex).expect("公钥解码失败");
            let signature = Signature::from_bytes(&sig_bytes).expect("签名转换失败");
            let public_key = PublicKey::from_bytes(&pub_key_bytes).expect("公钥转换失败");
            Crypto::verify_signature(&self.hash, &signature, &public_key)  // 使用哈希值验证签名
        } else {
            false
        }
    }
}