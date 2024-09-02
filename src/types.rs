use serde::{Deserialize, Serialize};
use zephyr_sdk::{
    prelude::*,
    soroban_sdk::{self, contracttype, xdr::ScVal, Address},
    DatabaseDerive, EnvClient,
};

#[derive(DatabaseDerive, Clone, Serialize)]
#[with_name("pools")]
pub struct PoolsTable {
    pub address: String,
}

#[derive(Serialize)]
pub struct Response {
    pub tx: String,
}

#[derive(Serialize)]
pub struct PoolData {
    pub address: String,
    pub token_id: String,
    pub genesis_period: i32,
    pub periods: i32,
    pub oracle: String,
    pub symbol: String,
    pub external: bool,
    pub oracle_asset: String,
    pub volatility: i128,
    pub admin: String,
    pub multiplier: i32,
    pub tot_liquidity: i128,
    pub tot_supply: i128,
    pub refund_global: i128,
}

#[derive(Clone, Copy)]
#[contracttype]
pub enum InstanceDataKey {
    TokenId,
    GenesisPeriod,
    Periods,
    Oracle,
    Symbol,
    External,
    OracleAsset,
    Volatility,
    Admin,
    Multiplier,
}

#[derive(Clone)]
#[contracttype]
pub struct BalanceObject {
    address: Address,
    period: i32,
}

#[derive(Clone)]
#[contracttype]
pub enum PersistentDataKey {
    Balance(BalanceObject),
    Principal(BalanceObject),
    TotLiquidity(i32),
    TotSupply(i32),
    FeePerShareUniversal(i32),
    FeePerShareParticular(BalanceObject),
    MaturedFeesParticular(BalanceObject),
    RefundParticular(BalanceObject),
    RefundGlobal(i32),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SimulateDeposit {
    pub contract: String,
    pub from: String,
    pub amount: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SimulateUpdateFeeRewards {
    pub contract: String,
    pub from: String,
    pub period: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SimulateWithdrawMatured {
    pub contract: String,
    pub from: String,
    pub period: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SimulateWithdraw {
    pub contract: String,
    pub from: String,
    pub period: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SimulateSubscribe {
    pub contract: String,
    pub from: String,
    pub amount: i128,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SimulateClaimReward {
    pub contract: String,
    pub from: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum SimulationRequest {
    Deposit(SimulateDeposit),
    UpdateFeeRewards(SimulateUpdateFeeRewards),
    WithdrawMatured(SimulateWithdrawMatured),
    Withdraw(SimulateWithdraw),
    Subscribe(SimulateSubscribe),
    ClaimReward(SimulateClaimReward),
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
