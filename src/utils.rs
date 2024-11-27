use sha2::{Digest, Sha256};
use crate::transaction::Transaction;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn current_timestamp() -> u64 {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).expect("时间戳生成失败！");
    since_epoch.as_secs()
}

pub fn calculate_merkle_root(transactions: &[Transaction]) -> String {
    if transactions.is_empty() {
        return String::from("0");
    }

    let mut hashes: Vec<String> = transactions
        .iter()
        .map(|tx| {
            let tx_data = format!("{:?}", tx);
            let mut hasher = Sha256::new();
            hasher.update(tx_data);
            hex::encode(hasher.finalize())
        })
        .collect();

    while hashes.len() > 1 {
        let mut new_level = Vec::new();
        for i in (0..hashes.len()).step_by(2) {
            let left = &hashes[i];
            let right = if i + 1 < hashes.len() {
                &hashes[i + 1]
            } else {
                left
            };

            let combined = format!("{}{}", left, right);
            let mut hasher = Sha256::new();
            hasher.update(combined);
            new_level.push(hex::encode(hasher.finalize()));
        }
        hashes = new_level;
    }

    hashes[0].clone()
}