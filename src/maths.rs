use fixed_point_math::{FixedPoint, STROOP};
use zephyr_sdk::soroban_sdk::Env;

pub(crate) fn calculate_period(current: i128, genesis: i128, periods: i128) -> i32 {
    let current_ledger = current;
    let genesis_ledger = genesis;
    let diff = current_ledger - genesis_ledger;
    let div = diff.fixed_div_ceil(periods, 1).unwrap();

    if div == 0 {
        return 1;
    }

    div as i32
}

// gives the current period (ex: 1)
// used
pub(crate) fn actual_period(e: &Env, genesis_ledger: i32, periods: i32) -> i32 {
    let current_ledger = e.ledger().sequence();
    calculate_period(
        current_ledger as i128,
        genesis_ledger as i128,
        periods as i128,
    )
}
