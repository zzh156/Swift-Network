use super::{ExecutionError, ExecutionResult};
use std::ops::{Add, Sub};

/// Gas unit
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GasUnit(u64);

impl GasUnit {
    /// Create new gas unit
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// Get value
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Add for GasUnit {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Sub for GasUnit {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

/// Gas schedule
#[derive(Debug, Clone)]
pub struct GasSchedule {
    /// Base computation cost
    pub computation_cost: GasUnit,
    /// Base storage cost
    pub storage_cost: GasUnit,
    /// Event emission cost
    pub event_cost: GasUnit,
    /// Cross-contract call cost
    pub cross_contract_call_cost: GasUnit,
}

impl Default for GasSchedule {
    fn default() -> Self {
        Self {
            computation_cost: GasUnit::new(1),
            storage_cost: GasUnit::new(10),
            event_cost: GasUnit::new(5),
            cross_contract_call_cost: GasUnit::new(20),
        }
    }
}

/// Gas status
pub struct GasStatus {
    /// Gas schedule
    schedule: GasSchedule,
    /// Gas limit
    limit: GasUnit,
    /// Gas used
    used: GasUnit,
}

impl GasStatus {
    /// Create new gas status
    pub fn new(schedule: GasSchedule, limit: GasUnit) -> Self {
        Self {
            schedule,
            limit,
            used: GasUnit::new(0),
        }
    }

    /// Deduct gas
    pub fn deduct_gas(&mut self, amount: GasUnit) -> ExecutionResult<()> {
        let new_used = self.used + amount;
        if new_used > self.limit {
            return Err(ExecutionError::GasError("Out of gas".into()));
        }
        self.used = new_used;
        Ok(())
    }

    /// Get remaining gas
    pub fn remaining_gas(&self) -> GasUnit {
        self.limit - self.used
    }

    /// Get gas used
    pub fn gas_used(&self) -> GasUnit {
        self.used
    }

    /// Charge computation
    pub fn charge_computation(&mut self, units: u64) -> ExecutionResult<()> {
        self.deduct_gas(self.schedule.computation_cost * units)
    }

    /// Charge storage
    pub fn charge_storage(&mut self, size: u64) -> ExecutionResult<()> {
        self.deduct_gas(self.schedule.storage_cost * size)
    }

    /// Charge event
    pub fn charge_event(&mut self) -> ExecutionResult<()> {
        self.deduct_gas(self.schedule.event_cost)
    }

    /// Charge cross-contract call
    pub fn charge_cross_contract_call(&mut self) -> ExecutionResult<()> {
        self.deduct_gas(self.schedule.cross_contract_call_cost)
    }
}