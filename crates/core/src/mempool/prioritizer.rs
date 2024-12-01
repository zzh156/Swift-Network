use crate::protocol::SignedTransaction;
use std::cmp::Ordering;

/// Transaction priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Priority(u64);

/// Transaction prioritizer
pub struct TransactionPrioritizer {
    // 可以添加配置参数
}

impl TransactionPrioritizer {
    pub fn new() -> Self {
        Self {}
    }

    /// Calculate transaction priority based on:
    /// 1. Gas price
    /// 2. Transaction size
    /// 3. Account nonce
    /// 4. Dependencies
    pub fn calculate_priority(&self, transaction: &SignedTransaction) -> Priority {
        let gas_price = transaction.gas_price();
        let size = transaction.encoded_size();
        
        // 基础优先级计算
        let base_priority = gas_price.saturating_mul(1_000_000) / size as u64;
        
        // 可以添加更多优先级因素
        Priority(base_priority)
    }

    /// Compare two transactions for ordering
    pub fn compare(
        &self,
        tx1: &SignedTransaction,
        tx2: &SignedTransaction,
    ) -> Ordering {
        let p1 = self.calculate_priority(tx1);
        let p2 = self.calculate_priority(tx2);
        p1.cmp(&p2)
    }
}