use super::{
    Consensus, ConsensusState, Proposal, Round, Vote,
    narwhal::NarwhalConsensus, types::Certificate,
};
use crate::protocol::{ProtocolError, ProtocolResult};
use std::sync::Arc;
use tokio::sync::RwLock;

/// BullShark consensus configuration
#[derive(Debug, Clone)]
pub struct BullSharkConfig {
    /// Base Narwhal configuration
    pub narwhal_config: NarwhalConfig,
    /// Commit wait time (ms)
    pub commit_wait_ms: u64,
    /// Reputation threshold
    pub reputation_threshold: f64,
}

/// BullShark consensus implementation
pub struct BullShark {
    /// Inner Narwhal consensus
    narwhal: Arc<NarwhalConsensus>,
    /// Configuration
    config: BullSharkConfig,
    /// Validator reputations
    reputations: Arc<RwLock<HashMap<String, f64>>>,
}

impl BullShark {
    pub fn new(
        config: BullSharkConfig,
        safety_rules: Arc<SafetyRules>,
    ) -> Self {
        let narwhal = Arc::new(NarwhalConsensus::new(
            config.narwhal_config.clone(),
            safety_rules,
        ));

        Self {
            narwhal,
            config,
            reputations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Update validator reputation
    async fn update_reputation(&self, validator: &str, score: f64) {
        let mut reputations = self.reputations.write().await;
        let current = reputations.entry(validator.to_string()).or_insert(1.0);
        *current = (*current + score).max(0.0).min(1.0);
    }

    /// Check if validator has sufficient reputation
    async fn has_sufficient_reputation(&self, validator: &str) -> bool {
        let reputations = self.reputations.read().await;
        reputations.get(validator)
            .map(|score| *score >= self.config.reputation_threshold)
            .unwrap_or(false)
    }

    /// Process proposal with reputation check
    async fn process_proposal_with_reputation(
        &self,
        proposal: Proposal,
    ) -> ProtocolResult<()> {
        // Check proposer reputation
        if !self.has_sufficient_reputation(&proposal.author).await {
            return Err(ProtocolError::InvalidProposal(
                "Insufficient reputation".into()
            ));
        }

        // Process with Narwhal
        let result = self.narwhal.process_proposal(proposal.clone()).await;

        // Update reputation based on result
        match result {
            Ok(_) => {
                self.update_reputation(&proposal.author, 0.1).await;
            }
            Err(_) => {
                self.update_reputation(&proposal.author, -0.2).await;
            }
        }

        result
    }
}

#[async_trait::async_trait]
impl Consensus for BullShark {
    async fn process_proposal(&self, proposal: Proposal) -> ProtocolResult<()> {
        self.process_proposal_with_reputation(proposal).await
    }

    async fn process_vote(&self, vote: Vote) -> ProtocolResult<()> {
        // Process vote and update reputation
        let result = self.narwhal.process_vote(vote.clone()).await;
        
        match result {
            Ok(_) => {
                self.update_reputation(&vote.author, 0.05).await;
            }
            Err(_) => {
                self.update_reputation(&vote.author, -0.1).await;
            }
        }

        result
    }

    fn current_round(&self) -> Round {
        self.narwhal.current_round()
    }

    fn state(&self) -> ConsensusState {
        self.narwhal.state()
    }
}