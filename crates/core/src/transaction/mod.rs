//! Transaction module for processing and managing transactions.

mod manager;
mod validator;

pub use manager::{TransactionManager, TransactionInfo};
pub use validator::{TransactionValidator, ValidationResult};

use crate::core::{Address, ObjectID};
use crate::crypto::{PublicKey, Signature};
use serde::{Serialize, Deserialize};

/// Transaction digest (32 bytes)
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionDigest([u8; 32]);

impl TransactionDigest {
    /// Create from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Transaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionData {
    /// Move transaction
    Move(MoveTransaction),
    /// System transaction
    System(SystemTransaction),
}

/// Move transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveTransaction {
    /// Module
    pub module: Option<MoveModule>,
    /// Function
    pub function: Option<MoveFunction>,
    /// Type arguments
    pub type_arguments: Vec<TypeTag>,
    /// Arguments
    pub arguments: Vec<Vec<u8>>,
}

/// Move module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveModule {
    /// Module bytecode
    pub bytecode: Vec<u8>,
    /// Module dependencies
    pub dependencies: Vec<ModuleId>,
}

/// Move function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveFunction {
    /// Function name
    pub name: String,
    /// Function visibility
    pub visibility: Visibility,
}

/// System transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemTransaction {
    /// Change epoch
    ChangeEpoch(EpochChange),
    /// Genesis
    Genesis(Genesis),
}

/// Transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction data
    pub data: TransactionData,
    /// Sender
    pub sender: Address,
    /// Gas budget
    pub gas_budget: u64,
    /// Gas price
    pub gas_price: u64,
    /// Dependencies
    pub dependencies: Vec<TransactionDigest>,
    /// Epoch
    pub epoch: u64,
    /// Expiration timestamp
    pub expiration: u64,
    /// Signature
    pub signature: Option<Signature>,
    /// Public key
    pub public_key: Option<PublicKey>,
}

impl Transaction {
    /// Create new transaction
    pub fn new(
        data: TransactionData,
        sender: Address,
        gas_budget: u64,
        gas_price: u64,
        dependencies: Vec<TransactionDigest>,
        epoch: u64,
        expiration: u64,
    ) -> Self {
        Self {
            data,
            sender,
            gas_budget,
            gas_price,
            dependencies,
            epoch,
            expiration,
            signature: None,
            public_key: None,
        }
    }

    /// Get transaction digest
    pub fn digest(&self) -> TransactionDigest {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(bincode::serialize(self).unwrap());
        TransactionDigest(hasher.finalize().into())
    }

    /// Sign transaction
    pub fn sign(&mut self, keypair: &KeyPair) {
        let signature = keypair.sign(self.digest().as_bytes());
        self.signature = Some(signature);
        self.public_key = Some(keypair.public());
    }

    /// Verify signature
    pub fn verify_signature(&self) -> bool {
        if let (Some(signature), Some(public_key)) = (&self.signature, &self.public_key) {
            public_key.verify(self.digest().as_bytes(), signature)
        } else {
            false
        }
    }

    /// Get gas budget
    pub fn gas_budget(&self) -> u64 {
        self.gas_budget
    }

    /// Get gas price
    pub fn gas_price(&self) -> u64 {
        self.gas_price
    }

    /// Get sender
    pub fn sender(&self) -> Address {
        self.sender
    }

    /// Get input objects
    pub fn input_objects(&self) -> Vec<ObjectID> {
        match &self.data {
            TransactionData::Move(move_tx) => {
                // Extract object references from Move transaction
                vec![] // TODO: Implement
            }
            TransactionData::System(_) => {
                // System transactions don't have input objects
                vec![]
            }
        }
    }
}