use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::borsh::maybestd::collections::HashSet;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedSet, Vector};
use near_sdk::json_types::{Base58CryptoHash, ValidAccountId, WrappedBalance, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::json;
use near_sdk::{
    assert_one_yocto, env, ext_contract, is_promise_success, log, near_bindgen, serde_json,
    AccountId, Balance, BorshStorageKey, CryptoHash, Gas, PanicOnDefault, PromiseOrValue,
    Timestamp,
};

pub mod callbacks;
pub mod ft_token_receiver;
pub mod internal;
pub mod lockup;
pub mod schedule;
pub mod termination;
pub mod util;
pub mod view;

use crate::lockup::*;
use crate::schedule::*;
use crate::termination::*;
use crate::util::*;

near_sdk::setup_alloc!();

pub type TimestampSec = u32;
pub type TokenAccountId = AccountId;

const GAS_FOR_FT_TRANSFER: Gas = 15_000_000_000_000;
const GAS_FOR_AFTER_FT_TRANSFER: Gas = 20_000_000_000_000;

const ONE_YOCTO: Balance = 1;
const NO_DEPOSIT: Balance = 0;

uint::construct_uint! {
    pub struct U256(4);
}

#[ext_contract(ext_self)]
pub trait SelfCallbacks {
    fn after_ft_transfer(
        &mut self,
        account_id: AccountId,
        lockup_claims: Vec<LockupClaim>,
    ) -> WrappedBalance;

    fn after_lockup_termination(
        &mut self,
        account_id: AccountId,
        amount: WrappedBalance,
    ) -> WrappedBalance;
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub token_account_id: TokenAccountId,

    pub lockups: Vector<Lockup>,

    pub account_lockups: LookupMap<AccountId, HashSet<LockupIndex>>,

    /// Account IDs that can create new lockups.
    pub deposit_whitelist: UnorderedSet<AccountId>,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    Lockups,
    AccountLockups,
    DepositWhitelist,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(token_account_id: ValidAccountId, deposit_whitelist: Vec<ValidAccountId>) -> Self {
        let mut deposit_whitelist_set = UnorderedSet::new(StorageKey::DepositWhitelist);
        deposit_whitelist_set.extend(deposit_whitelist.into_iter().map(|a| a.into()));
        Self {
            lockups: Vector::new(StorageKey::Lockups),
            account_lockups: LookupMap::new(StorageKey::AccountLockups),
            token_account_id: token_account_id.into(),
            deposit_whitelist: deposit_whitelist_set,
        }
    }

    pub fn claim(&mut self) -> PromiseOrValue<WrappedBalance> {
        panic!("Service is unavailable");
    }

    #[private]
    pub fn seize(&mut self, account_ids: Vec<AccountId>) -> WrappedBalance {
        let mut seized_balance: u128 = 0;

        for account_id in account_ids {
            if let Some(indices) = self.account_lockups.get(&account_id.clone()) {
                let mut account_balance: u128 = 0;

                for index in indices {
                    let lockup = self.lockups.get(index as u64).unwrap();

                    if lockup.claimed_balance > 0 {
                        continue;
                    }

                    account_balance += lockup.schedule.total_balance();

                    let zero_lockup = Lockup::new_unlocked(account_id.clone(), 0);
                    self.lockups.replace(index as _, &zero_lockup);
                }

                seized_balance += account_balance;
            }
        }

        let result = U128(seized_balance);

        log(json!({
            "event_type": "seize_for_batch",
            "amount": result,
        }));

        result
    }

    pub fn terminate(
        &mut self,
        lockup_index: LockupIndex,
        hashed_schedule: Option<Schedule>,
    ) -> PromiseOrValue<WrappedBalance> {
        let account_id = env::predecessor_account_id();
        let mut lockup = self
            .lockups
            .get(lockup_index as _)
            .expect("Lockup not found");
        let unvested_balance = lockup.terminate(&account_id, hashed_schedule);
        self.lockups.replace(lockup_index as _, &lockup);

        // no need to store empty lockup
        if lockup.schedule.total_balance() == 0 {
            let lockup_account_id: AccountId = lockup.account_id.into();
            let mut indices = self
                .account_lockups
                .get(&lockup_account_id)
                .unwrap_or_default();
            indices.remove(&lockup_index);
            self.internal_save_account_lockups(&lockup_account_id, indices);
        }

        if unvested_balance > 0 {
            ext_fungible_token::ft_transfer(
                account_id.clone(),
                unvested_balance.into(),
                Some(format!("Terminated lockup #{}", lockup_index)),
                &self.token_account_id,
                ONE_YOCTO,
                GAS_FOR_FT_TRANSFER,
            )
            .then(ext_self::after_lockup_termination(
                account_id,
                unvested_balance.into(),
                &env::current_account_id(),
                NO_DEPOSIT,
                GAS_FOR_AFTER_FT_TRANSFER,
            ))
            .into()
        } else {
            PromiseOrValue::Value(0.into())
        }
    }

    #[payable]
    pub fn add_to_deposit_whitelist(&mut self, account_id: ValidAccountId) {
        assert_one_yocto();
        self.assert_deposit_whitelist(&env::predecessor_account_id());
        self.deposit_whitelist.insert(account_id.as_ref());
    }

    #[payable]
    pub fn remove_from_deposit_whitelist(&mut self, account_id: ValidAccountId) {
        assert_one_yocto();
        self.assert_deposit_whitelist(&env::predecessor_account_id());
        self.deposit_whitelist.remove(account_id.as_ref());
    }
}
