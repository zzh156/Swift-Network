use std::collections::HashSet;
use serde::{Serialize, Deserialize};
use crate::protocol::{ProtocolError, ProtocolResult};

/// Object ability
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Ability {
    /// Object can be copied
    Copy,
    /// Object can be dropped
    Drop,
    /// Object can be stored
    Store,
    /// Object can be a key
    Key,
}

/// Object capabilities
#[derive(Debug, Clone, Default)]
pub struct ObjectCapabilities {
    /// Set of abilities
    abilities: HashSet<Ability>,
}

impl ObjectCapabilities {
    /// Create new capabilities
    pub fn new() -> Self {
        Self {
            abilities: HashSet::new(),
        }
    }

    /// Add ability
    pub fn add_ability(&mut self, ability: Ability) {
        self.abilities.insert(ability);
    }

    /// Remove ability
    pub fn remove_ability(&mut self, ability: &Ability) {
        self.abilities.remove(ability);
    }

    /// Check if has ability
    pub fn has_ability(&self, ability: &Ability) -> bool {
        self.abilities.contains(ability)
    }

    /// Get all abilities
    pub fn get_abilities(&self) -> &HashSet<Ability> {
        &self.abilities
    }

    /// Check if can be copied
    pub fn is_copy(&self) -> bool {
        self.has_ability(&Ability::Copy)
    }

    /// Check if can be dropped
    pub fn is_drop(&self) -> bool {
        self.has_ability(&Ability::Drop)
    }

    /// Check if can be stored
    pub fn is_store(&self) -> bool {
        self.has_ability(&Ability::Store)
    }

    /// Check if can be a key
    pub fn is_key(&self) -> bool {
        self.has_ability(&Ability::Key)
    }

    /// Verify abilities are valid
    pub fn verify(&self) -> ProtocolResult<()> {
        // Copy requires Drop
        if self.is_copy() && !self.is_drop() {
            return Err(ProtocolError::InvalidAbilities(
                "Copy requires Drop ability".into()
            ));
        }

        // Key requires Store
        if self.is_key() && !self.is_store() {
            return Err(ProtocolError::InvalidAbilities(
                "Key requires Store ability".into()
            ));
        }

        Ok(())
    }
}