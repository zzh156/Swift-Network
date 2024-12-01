use super::{errors::*, types::*};
use crate::crypto::{PublicKey, Signature};
use std::collections::HashSet;

pub struct CertificateBuilder {
    transaction: SignedTransaction,
    signatures: Vec<(PublicKey, Signature)>,
    signers: HashSet<PublicKey>,
}

impl CertificateBuilder {
    pub fn new(transaction: SignedTransaction) -> Self {
        Self {
            transaction,
            signatures: Vec::new(),
            signers: HashSet::new(),
        }
    }

    pub fn add_signature(
        &mut self,
        authority: PublicKey,
        signature: Signature,
    ) -> ProtocolResult<()> {
        // 检查是否已经有这个验证者的签名
        if self.signers.contains(&authority) {
            return Err(ProtocolError::InvalidCertificate(
                "Duplicate authority signature".into(),
            ));
        }

        // 验证签名
        if !signature.verify(&self.transaction.data, &authority) {
            return Err(ProtocolError::InvalidSignature(
                "Invalid authority signature".into(),
            ));
        }

        self.signatures.push((authority, signature));
        self.signers.insert(authority);
        Ok(())
    }

    pub fn build(self) -> ProtocolResult<TransactionCertificate> {
        // 检查是否有足够的签名
        if self.signatures.len() < 2 {  // 简化的示例，实际应该基于验证者权重
            return Err(ProtocolError::InvalidCertificate(
                "Insufficient signatures".into(),
            ));
        }

        Ok(TransactionCertificate {
            transaction: self.transaction,
            authority_signatures: self.signatures,
        })
    }
}

impl TransactionCertificate {
    pub fn verify(&self, committee: &Committee) -> ProtocolResult<()> {
        // 验证所有签名
        let mut weight = 0;
        for (authority, signature) in &self.authority_signatures {
            // 检查验证者是否在委员会中
            let auth_weight = committee.weight(authority)
                .ok_or_else(|| ProtocolError::InvalidCertificate(
                    "Authority not in committee".into(),
                ))?;

            // 验证签名
            if !signature.verify(&self.transaction.data, authority) {
                return Err(ProtocolError::InvalidSignature(
                    "Invalid authority signature".into(),
                ));
            }

            weight += auth_weight;
        }

        // 检查是否达到法定人数
        if weight < committee.quorum_threshold() {
            return Err(ProtocolError::InvalidCertificate(
                "Insufficient quorum".into(),
            ));
        }

        Ok(())
    }
}

// 委员会结构（简化版）
pub struct Committee {
    validators: HashMap<PublicKey, u64>, // 验证者 -> 权重
    total_weight: u64,
}

impl Committee {
    pub fn weight(&self, authority: &PublicKey) -> Option<u64> {
        self.validators.get(authority).copied()
    }

    pub fn quorum_threshold(&self) -> u64 {
        // 简化的 2/3 阈值计算
        (self.total_weight * 2) / 3 + 1
    }
}