use crate::*;

const START_UNLOCK_TIMESTAMP: u32 = 1694509200; // 2023-09-13T09:00:00 UTC
const FULL_UNLOCK_TIMESTAMP: u32 = 1757667600; // 2025-09-13T09:00:00 UTC

#[derive(Deserialize)]
#[serde(crate = "near_sdk::serde", tag = "type", content = "data", rename_all = "snake_case")]
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(Debug, PartialEq, Clone, Serialize)
)]
pub enum FtMessage {
    CreateLockup(Lockup),
    CreateInitialLockups(Vec<(ValidAccountId, U128)>)
}


#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert_eq!(
            env::predecessor_account_id(),
            self.token_account_id,
            "Invalid token ID"
        );

        self.assert_deposit_whitelist(sender_id.as_ref());
        let message: FtMessage =
            serde_json::from_str(&msg).expect("Expected FtMessage as msg");

        match message {
            FtMessage::CreateLockup(lockup) => {
                lockup.assert_new_valid(amount.0);
                let index = self.internal_add_lockup(&lockup);
                log!(
                    "Created new lockup for {} with index {}",
                    lockup.account_id.as_ref(),
                    index
                );
            }
            FtMessage::CreateInitialLockups(batch) => {
                let mut sum: u128 = 0;
                for (account_id, sweat) in batch {
                    let account_total = sweat.0;
                    sum = sum + account_total;

                    let user_lockup = Lockup {
                        account_id,
                        schedule: Schedule(vec![
                            Checkpoint {
                                timestamp: START_UNLOCK_TIMESTAMP,
                                balance: 0
                            },
                            Checkpoint {
                                timestamp: FULL_UNLOCK_TIMESTAMP,
                                balance: account_total,
                            },
                        ]),
                        claimed_balance: 0,
                        termination_config: None,
                    };
                    user_lockup.assert_new_valid(account_total);
                    let _index = self.internal_add_lockup(&user_lockup);
                }
                assert_eq!(amount.0, sum);
            }
        }

        PromiseOrValue::Value(0.into())
    }
}
