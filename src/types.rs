use serde::{Deserialize, Serialize};
use zephyr_sdk::{
    prelude::*,
    soroban_sdk::xdr::ScVal,
    DatabaseDerive, EnvClient,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct SimulateDeposit {
    pub from: String,
    pub amount: i64
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SimulateUpdateFeeRewards {
    pub from: String,
    pub period: i32
}

#[derive(Serialize, Deserialize, Clone)]
pub enum SimulationRequest {
    Deposit(SimulateDeposit),
    // UpdateFeeRewards(SimulateUpdateFeeRewards)
}

/*
pub enum SimulationRequest {
    Deposit(SimulateDeposit),
    UpdateFeeRewards(SimulateVote),
    WithdrawMarured(Simulate),
    Withdraw(Simulate),
    Subscribe(Simulate),
    ClaimReward(Simulate),
}
*/