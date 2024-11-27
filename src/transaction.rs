use serde::{Deserialize, Serialize};
use crate::crypto::Crypto;
use ed25519_dalek::{Keypair, PublicKey, Signature};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub sender: String,          // 交易发送方
    pub receiver: String,        // 交易接收方
    pub amount: u64,             // 交易金额
    pub fee: u64,                // 交易费用
    pub signature: Option<String>, // 签名
    pub public_key: Option<String>, // 公钥
}

impl Transaction {
    pub fn new(sender: String, receiver: String, amount: u64, fee: u64) -> Self {
        Transaction {
            sender,
            receiver,
            amount,
            fee,
            signature: None,
            public_key: None,
        }
    }

    // 为交易签名
    pub fn sign(&mut self, keypair: &Keypair) {
        let message = format!("{}{}{}", self.sender, self.receiver, self.amount);
        let signature = Crypto::sign_message(&message, keypair);  // 使用 Crypto 模块签名
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
            let message = format!("{}{}{}", self.sender, self.receiver, self.amount);
            Crypto::verify_signature(&message, &signature, &public_key)  // 使用 Crypto 模块验证签名
        } else {
            false
        }
    }
}
