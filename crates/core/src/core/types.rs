use serde::{Serialize, Deserialize};
use std::fmt;

/// Address type (20 bytes)
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Address([u8; 20]);

impl Address {
    /// Create from bytes
    pub fn from_bytes(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    /// Get bytes
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }
}

/// Sequence number
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct SequenceNumber(u64);

impl SequenceNumber {
    /// Create new sequence number
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// Get value
    pub fn value(&self) -> u64 {
        self.0
    }

    /// Increment
    pub fn increment(&mut self) {
        self.0 += 1;
    }
}

/// Balance type
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct Balance(u64);

impl Balance {
    /// Create new balance
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// Get value
    pub fn value(&self) -> u64 {
        self.0
    }

    /// Add amount
    pub fn add(&mut self, amount: u64) -> CoreResult<()> {
        self.0 = self.0.checked_add(amount)
            .ok_or_else(|| CoreError::InvalidBalance("Overflow".into()))?;
        Ok(())
    }

    /// Subtract amount
    pub fn sub(&mut self, amount: u64) -> CoreResult<()> {
        self.0 = self.0.checked_sub(amount)
            .ok_or_else(|| CoreError::InvalidBalance("Underflow".into()))?;
        Ok(())
    }
}

/// Coin type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coin {
    /// Coin type
    pub type_: TypeTag,
    /// Balance
    pub balance: Balance,
}

/// Type tag
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TypeTag {
    /// Bool
    Bool,
    /// U8
    U8,
    /// U64
    U64,
    /// U128
    U128,
    /// Address
    Address,
    /// Vector
    Vector(Box<TypeTag>),
    /// Struct
    Struct(StructTag),
}

/// Struct tag
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct StructTag {
    /// Address
    pub address: Address,
    /// Module
    pub module: String,
    /// Name
    pub name: String,
    /// Type parameters
    pub type_params: Vec<TypeTag>,
}