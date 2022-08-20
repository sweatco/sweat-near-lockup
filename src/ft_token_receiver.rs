use crate::*;

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
        let batched_users: BatchedUsers =
            serde_json::from_str(&msg).expect("Expected BatchedUsers as msg");

        let tge_timestamp = 1663070400; // 2022-09-13T12:00:00 UTC
        let full_unlock_timestamp = 1726228800; // 2024-09-13T12:00:00 UTC
        let mut sum: u128 = 0;
        for (account_id, sweat) in batched_users.batch {
            let account_total = sweat.0;
            sum = sum + account_total;

            let user_lockup = Lockup {
                account_id: account_id,
                schedule: Schedule(vec![
                    Checkpoint {
                        timestamp: tge_timestamp - 1,
                        balance: 0
                    },
                    Checkpoint {
                        timestamp: tge_timestamp,
                        balance: 10 * account_total / 100,
                    },
                    Checkpoint {
                        timestamp: full_unlock_timestamp,
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
        PromiseOrValue::Value(0.into())
    }
}
