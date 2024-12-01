use super::{Proposal, Round, Certificate};
use crate::protocol::{ProtocolError, ProtocolResult, TransactionDigest};
use std::collections::{HashMap, HashSet};

/// DAG node
#[derive(Debug, Clone)]
pub struct DagNode {
    /// The proposal
    pub proposal: Proposal,
    /// Children nodes
    pub children: HashSet<TransactionDigest>,
    /// Whether the node is committed
    pub committed: bool,
}

/// Directed acyclic graph
pub struct Dag {
    /// Nodes by digest
    nodes: HashMap<TransactionDigest, DagNode>,
    /// Nodes by round
    rounds: HashMap<Round, HashSet<TransactionDigest>>,
}

impl Dag {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            rounds: HashMap::new(),
        }
    }

    /// Add a new proposal to the DAG
    pub fn add_proposal(&mut self, proposal: Proposal) -> ProtocolResult<()> {
        let digest = proposal.digest();
        
        // Check if already exists
        if self.nodes.contains_key(&digest) {
            return Err(ProtocolError::InvalidProposal("Duplicate proposal".into()));
        }

        // Create node
        let node = DagNode {
            proposal: proposal.clone(),
            children: HashSet::new(),
            committed: false,
        };

        // Add to nodes
        self.nodes.insert(digest, node);

        // Add to rounds
        self.rounds
            .entry(proposal.round)
            .or_default()
            .insert(digest);

        // Update parent-child relationships
        for parent in &proposal.parents {
            if let Some(parent_node) = self.nodes.get_mut(parent) {
                parent_node.children.insert(digest);
            }
        }

        Ok(())
    }

    /// Find nodes that can be committed
    pub fn find_commit_candidates(&self) -> ProtocolResult<Vec<Proposal>> {
        let mut candidates = Vec::new();

        // Find nodes that satisfy Narwhal commit rules:
        // 1. All parents are committed
        // 2. Has enough children (2f + 1 in different rounds)
        for (digest, node) in &self.nodes {
            if node.committed {
                continue;
            }

            if self.can_commit(digest) {
                candidates.push(node.proposal.clone());
            }
        }

        Ok(candidates)
    }

    /// Check if a node can be committed
    fn can_commit(&self, digest: &TransactionDigest) -> bool {
        if let Some(node) = self.nodes.get(digest) {
            // Check parents
            for parent in &node.proposal.parents {
                if let Some(parent_node) = self.nodes.get(parent) {
                    if !parent_node.committed {
                        return false;
                    }
                }
            }

            // Check children
            let mut child_rounds = HashSet::new();
            for child in &node.children {
                if let Some(child_node) = self.nodes.get(child) {
                    child_rounds.insert(child_node.proposal.round);
                }
            }

            // Need 2f + 1 children in different rounds
            child_rounds.len() >= 3 // Simplified threshold
        } else {
            false
        }
    }
}