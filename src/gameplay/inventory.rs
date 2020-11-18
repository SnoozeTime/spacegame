use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InventoryError {
    #[error("Not enough cash")]
    NotEnoughCash,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Inventory {
    /// Amount of money
    scratch: u32,
}

impl Default for Inventory {
    fn default() -> Self {
        Self { scratch: 0 }
    }
}

impl Inventory {
    /// Remove all scratch. Happens at the end of each stage.
    pub fn drain_scratch(&mut self) {
        self.scratch = 0;
    }

    /// Add some money.
    /// Can never fail.
    pub fn add_scratch(&mut self, amt: u32) {
        self.scratch += amt;
    }

    pub fn scratch(&self) -> u32 {
        return self.scratch;
    }

    /// Remove some scratch. Returns an error if not enough scratch in the account.
    pub fn remove_scratch(&mut self, amt: u32) -> Result<(), InventoryError> {
        if amt > self.scratch {
            Err(InventoryError::NotEnoughCash)
        } else {
            self.scratch = self.scratch.saturating_sub(amt);
            Ok(())
        }
    }
}
