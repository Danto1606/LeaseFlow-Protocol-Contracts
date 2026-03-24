#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, Address, Env};

#[test]
fn test_lease_and_late_fees() {
    let env = Env::default();
    let contract_id = env.register(LeaseContract, ());
    let client = LeaseContractClient::new(&env, &contract_id);

    let landlord = Address::generate(&env);
    let tenant = Address::generate(&env);
    let amount = 1000i128;
    let grace_period_end = 100_000u64;
    let late_fee_flat = 20i128;
    let late_fee_per_day = 5i128;

    client.create_lease(
        &landlord,
        &tenant,
        &amount,
        &grace_period_end,
        &late_fee_flat,
        &late_fee_per_day,
    );

    let lease = client.get_lease();
    assert_eq!(lease.amount, 1000);
    assert_eq!(lease.debt, 0);

    // Time travels to 2 days later after grace period 
    // Wait, let's explicitly set the ledger
    // 2 days = 172800 secs. Let's add 176400 to make it exactly 2 full days and some balance.
    env.ledger().with_mut(|li| {
        li.timestamp = 100_000 + 176400; 
    });

    // Make a partial payment that covers part of the debt but no rent.
    // Flat fee $20 + (2 days * $5) = $30. Let's pay 25.
    client.pay_rent(&25);

    let updated_lease1 = client.get_lease();
    assert_eq!(updated_lease1.debt, 5);
    assert_eq!(updated_lease1.rent_paid, 0);
    assert_eq!(updated_lease1.flat_fee_applied, true);
    assert_eq!(updated_lease1.days_late_charged, 2);

    // Pay enough to clear the rest of debt and the full rent.
    // Remaining Debt = 5. Rent = 1000. Total = 1005.
    client.pay_rent(&1005);
    
    let updated_lease2 = client.get_lease();
    assert_eq!(updated_lease2.debt, 0);
    assert_eq!(updated_lease2.rent_paid, 0); // resets because rent was fully paid
    assert_eq!(updated_lease2.grace_period_end, 100_000 + 2592000);
    assert_eq!(updated_lease2.flat_fee_applied, false);
    assert_eq!(updated_lease2.days_late_charged, 0);
}
