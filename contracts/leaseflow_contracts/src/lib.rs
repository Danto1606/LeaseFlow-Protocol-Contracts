#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lease {
    pub landlord: Address,
    pub tenant: Address,
    pub amount: i128,
    pub active: bool,
    pub grace_period_end: u64,
    pub late_fee_flat: i128,
    pub late_fee_per_day: i128,
    pub debt: i128,
    pub flat_fee_applied: bool,
    pub days_late_charged: u64,
    pub rent_paid: i128,
}

#[contract]
pub struct LeaseContract;

#[contractimpl]
impl LeaseContract {
    /// Initializes a simple lease between a landlord and a tenant.
    pub fn create_lease(
        env: Env,
        landlord: Address,
        tenant: Address,
        amount: i128,
        grace_period_end: u64,
        late_fee_flat: i128,
        late_fee_per_day: i128,
    ) -> Symbol {
        let lease = Lease {
            landlord,
            tenant,
            amount,
            active: true,
            grace_period_end,
            late_fee_flat,
            late_fee_per_day,
            debt: 0,
            flat_fee_applied: false,
            days_late_charged: 0,
            rent_paid: 0,
        };
        env.storage()
            .instance()
            .set(&symbol_short!("lease"), &lease);
        symbol_short!("created")
    }

    /// Returns the current lease details stored in the contract.
    pub fn get_lease(env: Env) -> Lease {
        env.storage()
            .instance()
            .get(&symbol_short!("lease"))
            .expect("Lease not found")
    }

    /// Processes a rent payment, calculating and clearing debt before applying to rent.
    pub fn pay_rent(env: Env, payment_amount: i128) -> Symbol {
        let mut lease = Self::get_lease(env.clone());
        if !lease.active {
            panic!("Lease is not active");
        }

        let current_time = env.ledger().timestamp();

        // Calculate Debt
        if current_time > lease.grace_period_end {
            let seconds_late = current_time - lease.grace_period_end;
            
            if !lease.flat_fee_applied {
                lease.debt += lease.late_fee_flat;
                lease.flat_fee_applied = true;
            }

            let current_days_late = seconds_late / 86400; // Complete 24h periods
            if current_days_late > lease.days_late_charged {
                let newly_accrued_days = current_days_late - lease.days_late_charged;
                lease.debt += (newly_accrued_days as i128) * lease.late_fee_per_day;
                lease.days_late_charged = current_days_late;
            }
        }

        let mut remaining_payment = payment_amount;

        // Apply to debt first
        if lease.debt > 0 {
            if remaining_payment >= lease.debt {
                remaining_payment -= lease.debt;
                lease.debt = 0;
            } else {
                lease.debt -= remaining_payment;
                remaining_payment = 0;
            }
        }

        // Apply remainder to current month's rent
        if remaining_payment > 0 {
            lease.rent_paid += remaining_payment;
            
            // Advance month if fully paid
            if lease.rent_paid >= lease.amount {
                lease.rent_paid -= lease.amount;
                lease.grace_period_end += 2592000; // 30 days
                lease.flat_fee_applied = false;
                lease.days_late_charged = 0;
            }
        }

        env.storage().instance().set(&symbol_short!("lease"), &lease);
        symbol_short!("paid")
    }
}

mod test;
