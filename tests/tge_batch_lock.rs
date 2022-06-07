mod setup;

use crate::setup::*;

use near_sdk::json_types::WrappedBalance;
use near_sdk::json_types::U128;

#[test]
fn test_tge_user() {
    let e = Env::init(None);
    let users = Users::init(&e);
    let a = U128(d(60000, TOKEN_DECIMALS));
    let b = U128(d(3000, TOKEN_DECIMALS));
    let c = U128(d(25, TOKEN_DECIMALS));
    let tge_timestamp = 1654549079;
    let tge_plus_6_month = 1670349479;
    e.set_time_sec(tge_timestamp);

    let lockups = e.get_account_lockups(&users.alice);
    assert!(lockups.is_empty());

    let arr = vec![
        (users.alice.valid_account_id(), a),
        (users.bob.valid_account_id(), b),
        (users.charlie.valid_account_id(), c),
    ];
    let batch = BatchedUsers { batch: arr };
    let balance: WrappedBalance = e
        .add_batched_lockup(&e.owner, a.0 + b.0 + c.0, &batch)
        .unwrap_json();
    assert_eq!(balance.0, a.0 + b.0 + c.0);

    let lockups = e.get_account_lockups(&users.alice);
    assert_eq!(lockups.len(), 1);
    assert_eq!(lockups[0].1.total_balance, a.0);

    let lockups = e.get_account_lockups(&users.bob);
    assert_eq!(lockups.len(), 1);
    assert_eq!(lockups[0].1.total_balance, b.0);

    let lockups = e.get_account_lockups(&users.charlie);
    assert_eq!(lockups.len(), 1);
    assert_eq!(lockups[0].1.total_balance, c.0);

    e.set_time_sec(tge_plus_6_month);

    let lockups = e.get_account_lockups(&users.alice);
    assert_eq!(lockups.len(), 1);
    assert_eq!(lockups[0].1.unclaimed_balance, a.0);

    ft_storage_deposit(&users.alice, TOKEN_ID, &users.alice.account_id);
    ft_storage_deposit(&users.charlie, TOKEN_ID, &users.charlie.account_id);

    let res: WrappedBalance = e.claim(&users.alice).unwrap_json();
    assert_eq!(res.0, a.0);
    // User's lockups should be empty, since fully claimed.
    let lockups = e.get_account_lockups(&users.alice);
    assert!(lockups.is_empty());
    let balance = e.ft_balance_of(&users.alice);
    assert_eq!(balance, a.0);

    let res: WrappedBalance = e.claim(&users.charlie).unwrap_json();
    assert_eq!(res.0, c.0);
    // User's lockups should be empty, since fully claimed.
    let lockups = e.get_account_lockups(&users.charlie);
    assert!(lockups.is_empty());
    let balance = e.ft_balance_of(&users.charlie);
    assert_eq!(balance, c.0);
}
