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
}
