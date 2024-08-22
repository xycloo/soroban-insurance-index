use serde::{Deserialize, Serialize};
use zephyr_sdk::{
    prelude::*,
    soroban_sdk::xdr::ScVal,
    DatabaseDerive, EnvClient,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct SimulateDeposit {
    pub from: String,
    pub sequence: i64,
    pub amount: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SimulateVote {
    pub from: String,
    pub sequence: i64,
    pub hash: String,
    pub upvote: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum SimulationRequest {
    Deposit(SimulateVote),
    UpdateFeeRewards(SimulateVote),
    WithdrawMarured(Simulate),
    Withdraw(Simulate),
    Subscribe(Simulate),
    ClaimReward(Simulate),
}