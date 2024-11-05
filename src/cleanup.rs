use crate::lockup::Lockup;
use crate::schedule::{Checkpoint, Schedule};
use crate::*;
use near_sdk::json_types::ValidAccountId;
use near_sdk::near_bindgen;
use std::cmp;
use std::convert::TryFrom;

#[near_bindgen]
impl Contract {
    // Optimal batch size is 900
    #[private]
    pub fn clean(&mut self, batch_size: u64) -> u64 {
        let batch_size = cmp::min(batch_size, self.lockups.len());
        for _ in 0..batch_size {
            let lockup = self.lockups.pop().expect("Failed to pop lockup");
            self.account_lockups.remove(&lockup.account_id.into());
        }

        self.lockups.len()
    }
}
