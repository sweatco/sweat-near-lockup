use crate::*;
use std::collections::HashSet;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct ContractLegacy {
    pub token_account_id: TokenAccountId,

    pub lockups: Vector<Lockup>,

    pub account_lockups: LookupMap<AccountId, HashSet<LockupIndex>>,

    /// Account IDs that can create new lockups.
    pub deposit_whitelist: UnorderedSet<AccountId>,
}

#[near_bindgen]
impl Contract {
    #[private]
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        let old_state: ContractLegacy = env::state_read().expect("Failed to extract old contract state.");

        Contract {
            token_account_id: old_state.token_account_id,
            lockups: old_state.lockups,
            account_lockups: old_state.account_lockups,
            deposit_whitelist: old_state.deposit_whitelist,
            draft_operators_whitelist: UnorderedSet::new(StorageKey::DraftOperatorsWhitelist),
            next_draft_id: 0,
            drafts: LookupMap::new(StorageKey::Drafts),
            next_draft_group_id: 0,
            draft_groups: UnorderedMap::new(StorageKey::DraftGroups),
        }
    }
}
