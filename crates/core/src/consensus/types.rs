use serde::{Serialize, Deserialize};
use crate::protocol::{TransactionDigest, SignedTransaction};
use std::collections::HashSet;

/// Consensus round number
pub type Round = u64;

/// Proposal for a new batch of transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Round number
    pub round: Round,
    /// Author of the proposal
    pub author: String,
    /// Proposed transactions
    pub transactions: Vec<SignedTransaction>,
    /// Parents in the DAG
    pub parents: HashSet<TransactionDigest>,
    /// Signature
    pub signature: Vec<u8>,
}

/// Certificate for a committed batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    /// The proposal being certified
    pub proposal: Proposal,
    /// Signatures from validators
    pub signatures: Vec<(String, Vec<u8>)>,
}

/// Consensus state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusState {
    /// Current round
    pub round: Round,
    /// Last committed round
    pub last_committed_round: Round,
    /// Pending proposals
    pub pending_proposals: HashSet<TransactionDigest>,
    /// Committed certificates
    pub committed_certificates: Vec<Certificate>,
}