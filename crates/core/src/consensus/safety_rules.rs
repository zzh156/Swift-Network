use super::{Proposal, Vote, Round, Certificate};
use crate::protocol::{ProtocolError, ProtocolResult};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Safety rules configuration
#[derive(Debug, Clone)]
pub struct SafetyRulesConfig {
    /// Minimum quorum size
    pub quorum_size: usize,
    /// Maximum round gap
    pub max_round_gap: u64,
}

/// Safety rules for consensus
pub struct SafetyRules {
    /// Configuration
    config: SafetyRulesConfig,
    /// Highest voted round
    highest_voted_round: Arc<RwLock<Round>>,
    /// Highest certified round
    highest_certified_round: Arc<RwLock<Round>>,
    /// Locked round
    locked_round: Arc<RwLock<Option<Round>>>,
}

impl SafetyRules {
    pub fn new(config: SafetyRulesConfig) -> Self {
        Self {
            config,
            highest_voted_round: Arc::new(RwLock::new(0)),
            highest_certified_round: Arc::new(RwLock::new(0)),
            locked_round: Arc::new(RwLock::new(None)),
        }
    }

    /// Verify proposal safety rules
    pub fn verify_proposal(&self, proposal: &Proposal) -> ProtocolResult<()> {
        // Check round monotonicity
        let highest_voted = *self.highest_voted_round.blocking_read();
        if proposal.round <= highest_voted {
            return Err(ProtocolError::InvalidProposal(
                "Round not monotonic".into()
            ));
        }

        // Check round gap
        if proposal.round > highest_voted + self.config.max_round_gap {
            return Err(ProtocolError::InvalidProposal(
                "Round gap too large".into()
            ));
        }

        // Check locked round
        if let Some(locked) = *self.locked_round.blocking_read() {
            if proposal.round <= locked {
                return Err(ProtocolError::InvalidProposal(
                    "Violates locked round".into()
                ));
            }
        }

        Ok(())
    }

    /// Process vote
    pub async fn process_vote(&self, vote: Vote) -> ProtocolResult<()> {
        // Update highest voted round
        let mut highest_voted = self.highest_voted_round.write().await;
        if vote.round > *highest_voted {
            *highest_voted = vote.round;
        }

        // Check if we should lock the round
        if self.has_quorum_votes(&vote) {
            let mut locked = self.locked_round.write().await;
            *locked = Some(vote.round);
        }

        Ok(())
    }

    /// Sign proposal
    pub fn sign_proposal(&self, proposal: &Proposal) -> ProtocolResult<Vec<(String, Vec<u8>)>> {
        // Verify proposal first
        self.verify_proposal(proposal)?;

        // Generate signatures (simplified)
        let signatures = vec![
            ("validator1".to_string(), vec![1, 2, 3]),
            ("validator2".to_string(), vec![4, 5, 6]),
        ];

        Ok(signatures)
    }

    /// Check if we have quorum of votes
    fn has_quorum_votes(&self, vote: &Vote) -> bool {
        // Simplified quorum check
        vote.signatures.len() >= self.config.quorum_size
    }

    /// Update highest certified round
    pub async fn update_highest_certified_round(&self, round: Round) {
        let mut highest = self.highest_certified_round.write().await;
        if round > *highest {
            *highest = round;
        }
    }
}