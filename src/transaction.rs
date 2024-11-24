use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub sender: String,    // 交易发送方
    pub receiver: String,  // 交易接收方
    pub amount: u64,       // 交易金额
}

impl Transaction {
    pub fn new(sender: String, receiver: String, amount: u64) -> Self {
        Transaction {
            sender,
            receiver,
            amount,
        }
    }

    // 创建交易时，验证发送者是否有足够的 UTXO 支持这笔交易
    pub fn create_utxo_key(sender: &str, amount: u64) -> String {
        format!("{}:{}", sender, amount)
    }
}
