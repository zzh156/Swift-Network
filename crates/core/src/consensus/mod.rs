//! Consensus module implementing Narwhal-Bullshark consensus protocol.

mod narwhal;
mod bullshark;
mod dag;
mod safety_rules;
mod types;

pub use narwhal::{NarwhalConsensus, NarwhalConfig};
pub use bullshark::{BullShark, BullSharkConfig};
pub use dag::{Dag, DagNode, Round};
pub use safety_rules::{SafetyRules, Vote};
pub use types::{ConsensusState, Proposal, Certificate};

use crate::protocol::{ProtocolError, ProtocolResult};
use std::sync::Arc;

/// Consensus configuration
#[derive(Debug, Clone)]
pub struct ConsensusConfig {
    /// Consensus type (Narwhal or BullShark)
    pub consensus_type: ConsensusType,
    /// Minimum number of validators
    pub min_validators: usize,
    /// Block time target (ms)
    pub block_time_ms: u64,
    /// Maximum batch size
    pub max_batch_size: usize,
}

/// Consensus type
#[derive(Debug, Clone, Copy)]
pub enum ConsensusType {
    Narwhal,
    BullShark,
}

/// Main consensus interface
pub trait Consensus: Send + Sync {
    /// Process a new proposal
    async fn process_proposal(&self, proposal: Proposal) -> ProtocolResult<()>;
    
    /// Process a vote
    async fn process_vote(&self, vote: Vote) -> ProtocolResult<()>;
    
    /// Get current round
    fn current_round(&self) -> Round;
    
    /// Get consensus state
    fn state(&self) -> ConsensusState;
}