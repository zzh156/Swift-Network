use super::{Consensus, ConsensusState, Proposal, Round, Vote};
use crate::protocol::{ProtocolError, ProtocolResult};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Narwhal consensus configuration
#[derive(Debug, Clone)]
pub struct NarwhalConfig {
    /// Block time target (ms)
    pub block_time_ms: u64,
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Number of parents per proposal
    pub parents_count: usize,
}

/// Narwhal consensus implementation
pub struct NarwhalConsensus {
    /// Configuration
    config: NarwhalConfig,
    /// Current state
    state: Arc<RwLock<ConsensusState>>,
    /// Safety rules
    safety_rules: Arc<SafetyRules>,
    /// DAG
    dag: Arc<RwLock<Dag>>,
}

impl NarwhalConsensus {
    pub fn new(
        config: NarwhalConfig,
        safety_rules: Arc<SafetyRules>,
    ) -> Self {
        let state = ConsensusState {
            round: 0,
            last_committed_round: 0,
            pending_proposals: HashSet::new(),
            committed_certificates: Vec::new(),
        };

        Self {
            config,
            state: Arc::new(RwLock::new(state)),
            safety_rules,
            dag: Arc::new(RwLock::new(Dag::new())),
        }
    }

    /// Process a new proposal
    async fn process_proposal_internal(&self, proposal: Proposal) -> ProtocolResult<()> {
        // Verify proposal
        self.safety_rules.verify_proposal(&proposal)?;

        // Add to DAG
        let mut dag = self.dag.write().await;
        dag.add_proposal(proposal.clone())?;

        // Try to commit
        if let Some(certificates) = self.try_commit(&dag).await? {
            // Update state
            let mut state = self.state.write().await;
            state.committed_certificates.extend(certificates);
            state.last_committed_round = proposal.round;
        }

        Ok(())
    }

    /// Try to commit proposals
    async fn try_commit(&self, dag: &Dag) -> ProtocolResult<Option<Vec<Certificate>>> {
        // Find commit candidates using Narwhal rules
        let candidates = dag.find_commit_candidates()?;
        
        if candidates.is_empty() {
            return Ok(None);
        }

        // Create certificates
        let mut certificates = Vec::new();
        for proposal in candidates {
            let cert = self.create_certificate(proposal).await?;
            certificates.push(cert);
        }

        Ok(Some(certificates))
    }

    /// Create certificate for a proposal
    async fn create_certificate(&self, proposal: Proposal) -> ProtocolResult<Certificate> {
        // Collect signatures
        let signatures = self.safety_rules.sign_proposal(&proposal)?;
        
        Ok(Certificate {
            proposal,
            signatures,
        })
    }
}

#[async_trait::async_trait]
impl Consensus for NarwhalConsensus {
    async fn process_proposal(&self, proposal: Proposal) -> ProtocolResult<()> {
        self.process_proposal_internal(proposal).await
    }

    async fn process_vote(&self, vote: Vote) -> ProtocolResult<()> {
        self.safety_rules.process_vote(vote).await
    }

    fn current_round(&self) -> Round {
        self.state.blocking_read().round
    }

    fn state(&self) -> ConsensusState {
        self.state.blocking_read().clone()
    }
}