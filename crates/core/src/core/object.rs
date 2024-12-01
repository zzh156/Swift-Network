use super::{Address, CoreError, CoreResult, SequenceNumber, TypeTag};
use serde::{Serialize, Deserialize};
use std::fmt;

/// Object ID
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ObjectID([u8; 32]);

impl ObjectID {
    /// Create random object ID
    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 32];
        rng.fill(&mut bytes);
        Self(bytes)
    }

    /// Create from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Object owner
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Owner {
    /// Address owner
    AddressOwner(Address),
    /// Object owner
    ObjectOwner(ObjectID),
    /// Shared object
    Shared {
        initial_shared_version: SequenceNumber
    },
    /// Immutable object
    Immutable,
}

impl Owner {
    /// Get address if address owner
    pub fn get_address_owner(&self) -> Option<&Address> {
        match self {
            Self::AddressOwner(addr) => Some(addr),
            _ => None,
        }
    }

    /// Get object ID if object owner
    pub fn get_object_owner(&self) -> Option<&ObjectID> {
        match self {
            Self::ObjectOwner(id) => Some(id),
            _ => None,
        }
    }

    /// Check if shared
    pub fn is_shared(&self) -> bool {
        matches!(self, Self::Shared { .. })
    }

    /// Check if immutable
    pub fn is_immutable(&self) -> bool {
        matches!(self, Self::Immutable)
    }
}

/// Object data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    /// Object ID
    id: ObjectID,
    /// Object version
    version: SequenceNumber,
    /// Object owner
    owner: Owner,
    /// Object type
    type_: TypeTag,
    /// Object data
    data: Vec<u8>,
    /// Previous transaction
    previous_transaction: [u8; 32],
}

impl Object {
    /// Create new object
    pub fn new(
        id: ObjectID,
        owner: Owner,
        type_: TypeTag,
        data: Vec<u8>,
    ) -> Self {
        Self {
            id,
            version: SequenceNumber::new(0),
            owner,
            type_,
            data,
            previous_transaction: [0; 32],
        }
    }

    /// Get object ID
    pub fn id(&self) -> ObjectID {
        self.id
    }

    /// Get version
    pub fn version(&self) -> SequenceNumber {
        self.version
    }

    /// Get owner
    pub fn owner(&self) -> &Owner {
        &self.owner
    }

    /// Get type
    pub fn type_(&self) -> &TypeTag {
        &self.type_
    }

    /// Get data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get previous transaction
    pub fn previous_transaction(&self) -> &[u8; 32] {
        &self.previous_transaction
    }

    /// Set version
    pub fn set_version(&mut self, version: SequenceNumber) {
        self.version = version;
    }

    /// Set owner
    pub fn set_owner(&mut self, owner: Owner) {
        self.owner = owner;
    }

    /// Set data
    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }

    /// Set previous transaction
    pub fn set_previous_transaction(&mut self, tx: [u8; 32]) {
        self.previous_transaction = tx;
    }

    /// Check if owned by address
    pub fn is_owned_by(&self, address: &Address) -> bool {
        matches!(&self.owner, Owner::AddressOwner(addr) if addr == address)
    }

    /// Check if shared
    pub fn is_shared(&self) -> bool {
        self.owner.is_shared()
    }

    /// Check if immutable
    pub fn is_immutable(&self) -> bool {
        self.owner.is_immutable()
    }
}