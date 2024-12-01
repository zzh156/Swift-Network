use super::{StateError, StateResult};
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

/// Accumulator node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccumulatorNode {
    /// Node hash
    pub hash: [u8; 32],
    /// Left child hash
    pub left: Option<Box<AccumulatorNode>>,
    /// Right child hash
    pub right: Option<Box<AccumulatorNode>>,
}

impl AccumulatorNode {
    /// Create leaf node
    pub fn leaf(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update([0u8]); // Leaf prefix
        hasher.update(data);
        Self {
            hash: hasher.finalize().into(),
            left: None,
            right: None,
        }
    }

    /// Create internal node
    pub fn internal(left: AccumulatorNode, right: AccumulatorNode) -> Self {
        let mut hasher = Sha256::new();
        hasher.update([1u8]); // Internal prefix
        hasher.update(&left.hash);
        hasher.update(&right.hash);
        Self {
            hash: hasher.finalize().into(),
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }
    }
}

/// State accumulator
pub struct StateAccumulator {
    /// Root node
    root: Option<AccumulatorNode>,
    /// Leaf count
    leaf_count: usize,
}

impl StateAccumulator {
    /// Create new accumulator
    pub fn new() -> Self {
        Self {
            root: None,
            leaf_count: 0,
        }
    }

    /// Append leaf
    pub fn append(&mut self, data: &[u8]) -> StateResult<()> {
        let leaf = AccumulatorNode::leaf(data);
        match &mut self.root {
            None => {
                self.root = Some(leaf);
            }
            Some(root) => {
                // Find insertion position
                let mut current = root;
                let mut path = Vec::new();
                let mut pos = self.leaf_count;
                while pos > 0 {
                    path.push(current);
                    if pos % 2 == 0 {
                        current = current.left.as_mut().unwrap();
                    } else {
                        current = current.right.as_mut().unwrap();
                    }
                    pos /= 2;
                }

                // Insert leaf
                *current = leaf;

                // Update path
                for node in path.into_iter().rev() {
                    let left = node.left.take().unwrap();
                    let right = node.right.take().unwrap();
                    *node = AccumulatorNode::internal(*left, *right);
                }
            }
        }
        self.leaf_count += 1;
        Ok(())
    }

    /// Get root hash
    pub fn root_hash(&self) -> Option<[u8; 32]> {
        self.root.as_ref().map(|node| node.hash)
    }

    /// Get proof for leaf
    pub fn get_proof(&self, index: usize) -> StateResult<Vec<[u8; 32]>> {
        if index >= self.leaf_count {
            return Err(StateError::InvalidState("Invalid leaf index".into()));
        }

        let mut proof = Vec::new();
        let mut current = self.root.as_ref().unwrap();
        let mut pos = index;

        while pos > 0 {
            if pos % 2 == 0 {
                proof.push(current.right.as_ref().unwrap().hash);
                current = current.left.as_ref().unwrap();
            } else {
                proof.push(current.left.as_ref().unwrap().hash);
                current = current.right.as_ref().unwrap();
            }
            pos /= 2;
        }

        Ok(proof)
    }

    /// Verify proof
    pub fn verify_proof(
        root_hash: [u8; 32],
        leaf_data: &[u8],
        proof: &[[u8; 32]],
        index: usize,
    ) -> bool {
        let mut current_hash = AccumulatorNode::leaf(leaf_data).hash;
        let mut pos = index;

        for sibling in proof {
            let mut hasher = Sha256::new();
            hasher.update([1u8]); // Internal prefix
            if pos % 2 == 0 {
                hasher.update(&current_hash);
                hasher.update(sibling);
            } else {
                hasher.update(sibling);
                hasher.update(&current_hash);
            }
            current_hash = hasher.finalize().into();
            pos /= 2;
        }

        current_hash == root_hash
    }
}