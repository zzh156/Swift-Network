use super::{SystemError, SystemResult};
use crate::core::{Address, ObjectID};
use crate::storage::Storage;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Governance configuration
#[derive(Debug, Clone)]
pub struct GovernanceConfig {
    /// Minimum proposal deposit
    pub min_proposal_deposit: u64,
    /// Voting period (in seconds)
    pub voting_period: u64,
    /// Minimum participation rate
    pub min_participation_rate: f64,
    /// Required approval rate
    pub required_approval_rate: f64,
}

/// Proposal type
#[derive(Debug, Clone)]
pub enum ProposalType {
    /// Parameter update
    ParameterUpdate {
        parameter: String,
        value: String,
    },
    /// System upgrade
    SystemUpgrade {
        version: String,
        modules: Vec<Vec<u8>>,
    },
    /// Validator set update
    ValidatorSetUpdate {
        added: Vec<ValidatorInfo>,
        removed: Vec<Address>,
    },
    /// Custom proposal
    Custom {
        type_: String,
        data: Vec<u8>,
    },
}

/// Proposal status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposalStatus {
    /// Pending
    Pending,
    /// Active
    Active,
    /// Passed
    Passed,
    /// Failed
    Failed,
    /// Executed
    Executed,
}

/// Proposal
#[derive(Debug, Clone)]
pub struct Proposal {
    /// Proposal ID
    pub id: ObjectID,
    /// Proposer
    pub proposer: Address,
    /// Proposal type
    pub type_: ProposalType,
    /// Description
    pub description: String,
    /// Start time
    pub start_time: u64,
    /// End time
    pub end_time: u64,
    /// Status
    pub status: ProposalStatus,
    /// Deposit amount
    pub deposit: u64,
    /// Yes votes
    pub yes_votes: u64,
    /// No votes
    pub no_votes: u64,
    /// Voters
    pub voters: Vec<Address>,
}

/// Voting power
#[derive(Debug, Clone)]
pub struct VotingPower {
    /// Address
    pub address: Address,
    /// Power
    pub power: u64,
}

/// Governance system
pub struct Governance {
    /// Configuration
    config: GovernanceConfig,
    /// Storage
    storage: Arc<dyn Storage>,
    /// Proposals
    proposals: RwLock<HashMap<ObjectID, Proposal>>,
    /// Voting powers
    voting_powers: RwLock<HashMap<Address, u64>>,
}

impl Governance {
    /// Create new governance system
    pub fn new(
        config: GovernanceConfig,
        storage: Arc<dyn Storage>,
    ) -> Self {
        Self {
            config,
            storage,
            proposals: RwLock::new(HashMap::new()),
            voting_powers: RwLock::new(HashMap::new()),
        }
    }

    /// Initialize governance system
    pub async fn initialize(&mut self) -> SystemResult<()> {
        // Load proposals
        let proposals = self.storage.get_proposals().await
            .map_err(|e| SystemError::GovernanceError(e.to_string()))?;
        *self.proposals.write().await = proposals;

        // Load voting powers
        let voting_powers = self.storage.get_voting_powers().await
            .map_err(|e| SystemError::GovernanceError(e.to_string()))?;
        *self.voting_powers.write().await = voting_powers;

        Ok(())
    }

    /// Create proposal
    pub async fn create_proposal(
        &self,
        proposer: Address,
        type_: ProposalType,
        description: String,
        deposit: u64,
    ) -> SystemResult<ObjectID> {
        // Validate deposit
        if deposit < self.config.min_proposal_deposit {
            return Err(SystemError::GovernanceError("Insufficient deposit".into()));
        }

        // Create proposal
        let proposal = Proposal {
            id: ObjectID::random(),
            proposer,
            type_,
            description,
            start_time: crate::utils::current_timestamp(),
            end_time: crate::utils::current_timestamp() + self.config.voting_period,
            status: ProposalStatus::Active,
            deposit,
            yes_votes: 0,
            no_votes: 0,
            voters: Vec::new(),
        };

        // Store proposal
        self.storage.put_proposal(&proposal).await
            .map_err(|e| SystemError::GovernanceError(e.to_string()))?;
        self.proposals.write().await.insert(proposal.id, proposal.clone());

        Ok(proposal.id)
    }

    /// Vote on proposal
    pub async fn vote(
        &self,
        proposal_id: ObjectID,
        voter: Address,
        approve: bool,
    ) -> SystemResult<()> {
        // Get proposal
        let mut proposals = self.proposals.write().await;
        let proposal = proposals.get_mut(&proposal_id)
            .ok_or_else(|| SystemError::GovernanceError("Proposal not found".into()))?;

        // Check status
        if proposal.status != ProposalStatus::Active {
            return Err(SystemError::GovernanceError("Proposal not active".into()));
        }

        // Check if already voted
        if proposal.voters.contains(&voter) {
            return Err(SystemError::GovernanceError("Already voted".into()));
        }

        // Get voting power
        let voting_power = self.voting_powers.read().await.get(&voter)
            .copied()
            .unwrap_or(0);

        // Update votes
        if approve {
            proposal.yes_votes += voting_power;
        } else {
            proposal.no_votes += voting_power;
        }
        proposal.voters.push(voter);

        // Store updated proposal
        self.storage.put_proposal(proposal).await
            .map_err(|e| SystemError::GovernanceError(e.to_string()))?;

        Ok(())
    }

    /// Execute proposal
    pub async fn execute_proposal(&self, proposal_id: ObjectID) -> SystemResult<()> {
        // Get proposal
        let mut proposals = self.proposals.write().await;
        let proposal = proposals.get_mut(&proposal_id)
            .ok_or_else(|| SystemError::GovernanceError("Proposal not found".into()))?;

        // Check status
        if proposal.status != ProposalStatus::Active {
            return Err(SystemError::GovernanceError("Proposal not active".into()));
        }

        // Check if voting period ended
        if crate::utils::current_timestamp() < proposal.end_time {
            return Err(SystemError::GovernanceError("Voting period not ended".into()));
        }

        // Calculate participation and approval rates
        let total_votes = proposal.yes_votes + proposal.no_votes;
        let total_power: u64 = self.voting_powers.read().await.values().sum();
        let participation_rate = total_votes as f64 / total_power as f64;
        let approval_rate = proposal.yes_votes as f64 / total_votes as f64;

        // Check rates
        if participation_rate < self.config.min_participation_rate {
            proposal.status = ProposalStatus::Failed;
        } else if approval_rate < self.config.required_approval_rate {
            proposal.status = ProposalStatus::Failed;
        } else {
            proposal.status = ProposalStatus::Passed;
        }

        // Execute if passed
        if proposal.status == ProposalStatus::Passed {
            self.execute_proposal_type(&proposal.type_).await?;
            proposal.status = ProposalStatus::Executed;
        }

        // Store updated proposal
        self.storage.put_proposal(proposal).await
            .map_err(|e| SystemError::GovernanceError(e.to_string()))?;

        Ok(())
    }

    /// Execute proposal type
    async fn execute_proposal_type(&self, type_: &ProposalType) -> SystemResult<()> {
        match type_ {
            ProposalType::ParameterUpdate { parameter, value } => {
                self.storage.update_parameter(parameter, value).await
                    .map_err(|e| SystemError::GovernanceError(e.to_string()))?;
            }
            ProposalType::SystemUpgrade { version, modules } => {
                self.storage.upgrade_system(version, modules).await
                    .map_err(|e| SystemError::GovernanceError(e.to_string()))?;
            }
            ProposalType::ValidatorSetUpdate { added, removed } => {
                self.storage.update_validator_set(added, removed).await
                    .map_err(|e| SystemError::GovernanceError(e.to_string()))?;
            }
            ProposalType::Custom { type_, data } => {
                self.storage.execute_custom_proposal(type_, data).await
                    .map_err(|e| SystemError::GovernanceError(e.to_string()))?;
            }
        }
        Ok(())
    }
}