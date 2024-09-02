use maths::*;
use serde::Serialize;
use types::*;
use zephyr_sdk::{
    prelude::*,
    soroban_sdk::{
        self, vec,
        xdr::{
            self, LedgerEntryData, ScVal, TransactionEnvelope, TransactionExt,
            TransactionV1Envelope,
        },
        Address, Bytes, BytesN, IntoVal, String as SString, Symbol,
    },
    utils::address_to_alloc_string,
    DatabaseDerive, EnvClient,
};

mod maths;
mod types;

const CONTRACT_ADDRESS: [u8; 32] = [
    49, 27, 135, 97, 127, 42, 250, 76, 254, 105, 64, 142, 243, 103, 117, 92, 63, 2, 173, 226, 148,
    9, 73, 17, 217, 128, 179, 107, 100, 175, 71, 9,
];

#[no_mangle]
pub extern "C" fn on_close() {
    let env = EnvClient::new();
    for event in env.reader().pretty().soroban_events() {
        let action: Symbol = env.from_scval(&event.topics[0]);
        if action == Symbol::new(&env.soroban(), "deployed") {
            let address: Address = env.from_scval(&event.data);
            let address_string = address_to_alloc_string(&env, address);

            let pool = PoolsTable {
                address: address_string.clone(),
            };
            env.log()
                .debug(format!("New address {}", address_string), None);
            pool.put(&env);
        }
    }
}

// create a function to get the specific data about a certain user

#[no_mangle]
pub extern "C" fn get_pools() {
    let env = EnvClient::empty();

    // soroban env to get the latest ledger
    let soroban_env = env.soroban();

    let pools = env.read::<PoolsTable>();
    let addresses: Vec<String> = pools.iter().map(|pool| pool.address.clone()).collect();

    let pool_data: Vec<PoolData> = addresses
        .iter()
        .filter_map(|address| {
            let instance = env
                .read_contract_instance(stellar_strkey::Contract::from_string(address).unwrap().0)
                .unwrap()
                .unwrap();
            let LedgerEntryData::ContractData(data) = instance.entry.data else {
                panic!("Expected ContractData");
            };
            let ScVal::ContractInstance(instance) = data.val else {
                panic!()
            };

            let token_id_scval = &instance
                .storage
                .as_ref()
                .unwrap()
                .iter()
                .find(|entry| entry.key == env.to_scval(InstanceDataKey::TokenId))
                .unwrap()
                .val
                .clone();
            let genesis_priod_scval = &instance
                .storage
                .as_ref()
                .unwrap()
                .iter()
                .find(|entry| entry.key == env.to_scval(InstanceDataKey::GenesisPeriod))
                .unwrap()
                .val
                .clone();
            let periods_scval = &instance
                .storage
                .as_ref()
                .unwrap()
                .iter()
                .find(|entry| entry.key == env.to_scval(InstanceDataKey::Periods))
                .unwrap()
                .val
                .clone();
            let oracle_scval = &instance
                .storage
                .as_ref()
                .unwrap()
                .iter()
                .find(|entry| entry.key == env.to_scval(InstanceDataKey::Oracle))
                .unwrap()
                .val
                .clone();
            let symbol_scval = &instance
                .storage
                .as_ref()
                .unwrap()
                .iter()
                .find(|entry| entry.key == env.to_scval(InstanceDataKey::Symbol))
                .unwrap()
                .val
                .clone();
            let external_scval = &instance
                .storage
                .as_ref()
                .unwrap()
                .iter()
                .find(|entry| entry.key == env.to_scval(InstanceDataKey::External))
                .unwrap()
                .val
                .clone();
            let oracle_asset_scval = &instance
                .storage
                .as_ref()
                .unwrap()
                .iter()
                .find(|entry| entry.key == env.to_scval(InstanceDataKey::OracleAsset))
                .unwrap()
                .val
                .clone();
            let volatility_scval = &instance
                .storage
                .as_ref()
                .unwrap()
                .iter()
                .find(|entry| entry.key == env.to_scval(InstanceDataKey::Volatility))
                .unwrap()
                .val
                .clone();
            let admin_scval = &instance
                .storage
                .as_ref()
                .unwrap()
                .iter()
                .find(|entry| entry.key == env.to_scval(InstanceDataKey::Admin))
                .unwrap()
                .val
                .clone();
            let multiplier_scval = &instance
                .storage
                .as_ref()
                .unwrap()
                .iter()
                .find(|entry| entry.key == env.to_scval(InstanceDataKey::Multiplier))
                .unwrap()
                .val
                .clone();

            // convert from sc_vals
            let token_id = env.from_scval::<String>(token_id_scval);
            let genesis_period = env.from_scval::<i32>(genesis_priod_scval);
            let periods = env.from_scval::<i32>(periods_scval);
            let oracle = env.from_scval::<String>(oracle_scval);
            let symbol = env.from_scval::<String>(symbol_scval);
            let external = env.from_scval::<bool>(external_scval);
            let oracle_asset = env.from_scval::<String>(oracle_asset_scval);
            let volatility = env.from_scval::<i128>(volatility_scval);
            let admin = env.from_scval::<String>(admin_scval);
            let multiplier = env.from_scval::<i32>(multiplier_scval);

            let entries = env
                .read_contract_entries(stellar_strkey::Contract::from_string(address).unwrap().0)
                .unwrap();

            let period = actual_period(&soroban_env, genesis_period, periods);

            let mut tot_liquidity: i128 = 0;
            let mut tot_supply: i128 = 0;
            let mut refund_global: i128 = 0;

            for entry in entries.clone() {
                let LedgerEntryData::ContractData(data) = entry.entry.data else {
                    env.log()
                        .debug(format!("not contract data {:?}", entry.entry.data), None);
                    panic!()
                };

                if let Ok(entry_key) = env.try_from_scval::<PersistentDataKey>(&data.key) {
                    match entry_key {
                        PersistentDataKey::TotSupply(p) if p == period => {
                            tot_supply = env.from_scval(&data.val);
                            env.log().debug(
                                format!(
                                    "total supply for period {:?}: {:?}",
                                    period, tot_supply as i64
                                ),
                                None,
                            );
                        }
                        PersistentDataKey::TotLiquidity(p) if p == period => {
                            tot_liquidity = env.from_scval(&data.val);
                            env.log().debug(
                                format!(
                                    "total liquidity for period {:?}: {:?}",
                                    period, tot_liquidity as i64
                                ),
                                None,
                            );
                        }
                        PersistentDataKey::RefundGlobal(p) if p == period => {
                            refund_global = env.from_scval(&data.val);
                            env.log().debug(
                                format!(
                                    "total refund for period {:?}: {:?}",
                                    period, refund_global as i64
                                ),
                                None,
                            );
                        }
                        _ => (),
                    }
                }
            }
            Some(PoolData {
                address: address.clone(),
                token_id,
                genesis_period,
                periods,
                oracle,
                symbol,
                external,
                oracle_asset,
                volatility,
                admin,
                multiplier,
                tot_liquidity,
                tot_supply,
                refund_global,
            })
        })
        .collect();

    env.conclude(&pool_data)
}

#[no_mangle]
pub extern "C" fn simulate() {
    let env = EnvClient::empty();
    let request: SimulationRequest = env.read_request_body();
    //let new_sequence = env.soroban().ledger().sequence() + 1;

    let response = match request {
        SimulationRequest::Deposit(SimulateDeposit {
            contract,
            from,
            amount,
        }) => {
            let account = stellar_strkey::ed25519::PublicKey::from_string(&from)
                .unwrap()
                .0;
            let sequence = env
                .read_account_from_ledger(account)
                .unwrap()
                .unwrap()
                .seq_num;
            let new_sequence = sequence as i64 + 1;

            let address = Address::from_string(&SString::from_str(&env.soroban(), &from));

            // amount is i64: good like this?
            let tx = env
                .simulate_contract_call_to_tx(
                    from,
                    new_sequence,
                    stellar_strkey::Contract::from_string(&contract).unwrap().0,
                    Symbol::new(&env.soroban(), "deposit"),
                    vec![
                        &env.soroban(),
                        address.into_val(env.soroban()),
                        (amount as i128).into_val(env.soroban()),
                    ],
                )
                .unwrap()
                .tx
                .unwrap();

            // access the response objet and change the resources
            let tx = TransactionEnvelope::from_xdr_base64(tx, Limits::none()).unwrap();
            let TransactionEnvelope::Tx(TransactionV1Envelope { mut tx, .. }) = tx else {
                panic!()
            };
            let TransactionExt::V1(mut v1ext) = tx.ext else {
                panic!()
            };
            let mut r = v1ext.resources;
            r.read_bytes += 200;
            r.write_bytes += 100;
            v1ext.resource_fee += 100;
            v1ext.resources = r;
            tx.ext = TransactionExt::V1(v1ext);
            tx.fee += 100;

            // build again the simulated transaction after we updated the resources
            let envelope = TransactionEnvelope::Tx(TransactionV1Envelope {
                tx,
                signatures: std::vec![].try_into().unwrap(),
            });
            Response {
                tx: envelope.to_xdr_base64(Limits::none()).unwrap(),
            }
        }
        SimulationRequest::UpdateFeeRewards(SimulateUpdateFeeRewards {
            contract,
            from,
            period,
        }) => {
            let account = stellar_strkey::ed25519::PublicKey::from_string(&from)
                .unwrap()
                .0;
            let sequence = env
                .read_account_from_ledger(account)
                .unwrap()
                .unwrap()
                .seq_num;
            let new_sequence = sequence as i64 + 1;

            let address = Address::from_string(&SString::from_str(&env.soroban(), &from));

            let tx = env
                .simulate_contract_call_to_tx(
                    from,
                    new_sequence,
                    stellar_strkey::Contract::from_string(&contract).unwrap().0,
                    Symbol::new(&env.soroban(), "update_fee_rewards"),
                    vec![
                        &env.soroban(),
                        address.into_val(env.soroban()),
                        period.into_val(env.soroban()),
                    ],
                )
                .unwrap()
                .tx
                .unwrap();

            // access the response objet and change the resources
            let tx = TransactionEnvelope::from_xdr_base64(tx, Limits::none()).unwrap();
            let TransactionEnvelope::Tx(TransactionV1Envelope { mut tx, .. }) = tx else {
                panic!()
            };
            let TransactionExt::V1(mut v1ext) = tx.ext else {
                panic!()
            };
            let mut r = v1ext.resources;
            r.read_bytes += 200;
            r.write_bytes += 100;
            v1ext.resource_fee += 100;
            v1ext.resources = r;
            tx.ext = TransactionExt::V1(v1ext);
            tx.fee += 100;

            // build again the simulated transaction after we updated the resources
            let envelope = TransactionEnvelope::Tx(TransactionV1Envelope {
                tx,
                signatures: std::vec![].try_into().unwrap(),
            });
            Response {
                tx: envelope.to_xdr_base64(Limits::none()).unwrap(),
            }
        }
        SimulationRequest::WithdrawMatured(SimulateWithdrawMatured {
            contract,
            from,
            period,
        }) => {
            let account = stellar_strkey::ed25519::PublicKey::from_string(&from)
                .unwrap()
                .0;
            let sequence = env
                .read_account_from_ledger(account)
                .unwrap()
                .unwrap()
                .seq_num;
            let new_sequence = sequence as i64 + 1;

            let address = Address::from_string(&SString::from_str(&env.soroban(), &from));

            let tx = env
                .simulate_contract_call_to_tx(
                    from,
                    new_sequence,
                    stellar_strkey::Contract::from_string(&contract).unwrap().0,
                    Symbol::new(&env.soroban(), "withdraw_matured"),
                    vec![
                        &env.soroban(),
                        address.into_val(env.soroban()),
                        period.into_val(env.soroban()),
                    ],
                )
                .unwrap()
                .tx
                .unwrap();

            // access the response objet and change the resources
            let tx = TransactionEnvelope::from_xdr_base64(tx, Limits::none()).unwrap();
            let TransactionEnvelope::Tx(TransactionV1Envelope { mut tx, .. }) = tx else {
                panic!()
            };
            let TransactionExt::V1(mut v1ext) = tx.ext else {
                panic!()
            };
            let mut r = v1ext.resources;
            r.read_bytes += 200;
            r.write_bytes += 100;
            v1ext.resource_fee += 100;
            v1ext.resources = r;
            tx.ext = TransactionExt::V1(v1ext);
            tx.fee += 100;

            // build again the simulated transaction after we updated the resources
            let envelope = TransactionEnvelope::Tx(TransactionV1Envelope {
                tx,
                signatures: std::vec![].try_into().unwrap(),
            });
            Response {
                tx: envelope.to_xdr_base64(Limits::none()).unwrap(),
            }
        }
        SimulationRequest::WithdrawMatured(SimulateWithdrawMatured {
            contract,
            from,
            period,
        }) => {
            let account = stellar_strkey::ed25519::PublicKey::from_string(&from)
                .unwrap()
                .0;
            let sequence = env
                .read_account_from_ledger(account)
                .unwrap()
                .unwrap()
                .seq_num;
            let new_sequence = sequence as i64 + 1;

            let address = Address::from_string(&SString::from_str(&env.soroban(), &from));

            let tx = env
                .simulate_contract_call_to_tx(
                    from,
                    new_sequence,
                    stellar_strkey::Contract::from_string(&contract).unwrap().0,
                    Symbol::new(&env.soroban(), "withdraw_matured"),
                    vec![
                        &env.soroban(),
                        address.into_val(env.soroban()),
                        period.into_val(env.soroban()),
                    ],
                )
                .unwrap()
                .tx
                .unwrap();

            // access the response objet and change the resources
            let tx = TransactionEnvelope::from_xdr_base64(tx, Limits::none()).unwrap();
            let TransactionEnvelope::Tx(TransactionV1Envelope { mut tx, .. }) = tx else {
                panic!()
            };
            let TransactionExt::V1(mut v1ext) = tx.ext else {
                panic!()
            };
            let mut r = v1ext.resources;
            r.read_bytes += 200;
            r.write_bytes += 100;
            v1ext.resource_fee += 100;
            v1ext.resources = r;
            tx.ext = TransactionExt::V1(v1ext);
            tx.fee += 100;

            // build again the simulated transaction after we updated the resources
            let envelope = TransactionEnvelope::Tx(TransactionV1Envelope {
                tx,
                signatures: std::vec![].try_into().unwrap(),
            });
            Response {
                tx: envelope.to_xdr_base64(Limits::none()).unwrap(),
            }
        }
        SimulationRequest::Withdraw(SimulateWithdraw {
            contract,
            from,
            period,
        }) => {
            let account = stellar_strkey::ed25519::PublicKey::from_string(&from)
                .unwrap()
                .0;
            let sequence = env
                .read_account_from_ledger(account)
                .unwrap()
                .unwrap()
                .seq_num;
            let new_sequence = sequence as i64 + 1;

            let address = Address::from_string(&SString::from_str(&env.soroban(), &from));

            let tx = env
                .simulate_contract_call_to_tx(
                    from,
                    new_sequence,
                    stellar_strkey::Contract::from_string(&contract).unwrap().0,
                    Symbol::new(&env.soroban(), "withdraw"),
                    vec![
                        &env.soroban(),
                        address.into_val(env.soroban()),
                        period.into_val(env.soroban()),
                    ],
                )
                .unwrap()
                .tx
                .unwrap();

            // access the response objet and change the resources
            let tx = TransactionEnvelope::from_xdr_base64(tx, Limits::none()).unwrap();
            let TransactionEnvelope::Tx(TransactionV1Envelope { mut tx, .. }) = tx else {
                panic!()
            };
            let TransactionExt::V1(mut v1ext) = tx.ext else {
                panic!()
            };
            let mut r = v1ext.resources;
            r.read_bytes += 200;
            r.write_bytes += 100;
            v1ext.resource_fee += 100;
            v1ext.resources = r;
            tx.ext = TransactionExt::V1(v1ext);
            tx.fee += 100;

            // build again the simulated transaction after we updated the resources
            let envelope = TransactionEnvelope::Tx(TransactionV1Envelope {
                tx,
                signatures: std::vec![].try_into().unwrap(),
            });
            Response {
                tx: envelope.to_xdr_base64(Limits::none()).unwrap(),
            }
        }
        SimulationRequest::Subscribe(SimulateSubscribe {
            contract,
            from,
            amount,
        }) => {
            let account = stellar_strkey::ed25519::PublicKey::from_string(&from)
                .unwrap()
                .0;
            let sequence = env
                .read_account_from_ledger(account)
                .unwrap()
                .unwrap()
                .seq_num;
            let new_sequence = sequence as i64 + 1;

            let address = Address::from_string(&SString::from_str(&env.soroban(), &from));

            let tx = env
                .simulate_contract_call_to_tx(
                    from,
                    new_sequence,
                    stellar_strkey::Contract::from_string(&contract).unwrap().0,
                    Symbol::new(&env.soroban(), "subscribe"),
                    vec![
                        &env.soroban(),
                        address.into_val(env.soroban()),
                        amount.into_val(env.soroban()),
                    ],
                )
                .unwrap()
                .tx
                .unwrap();

            // access the response objet and change the resources
            let tx = TransactionEnvelope::from_xdr_base64(tx, Limits::none()).unwrap();
            let TransactionEnvelope::Tx(TransactionV1Envelope { mut tx, .. }) = tx else {
                panic!()
            };
            let TransactionExt::V1(mut v1ext) = tx.ext else {
                panic!()
            };
            let mut r = v1ext.resources;
            r.read_bytes += 200;
            r.write_bytes += 100;
            v1ext.resource_fee += 100;
            v1ext.resources = r;
            tx.ext = TransactionExt::V1(v1ext);
            tx.fee += 100;

            // build again the simulated transaction after we updated the resources
            let envelope = TransactionEnvelope::Tx(TransactionV1Envelope {
                tx,
                signatures: std::vec![].try_into().unwrap(),
            });
            Response {
                tx: envelope.to_xdr_base64(Limits::none()).unwrap(),
            }
        }
        SimulationRequest::ClaimReward(SimulateClaimReward { contract, from }) => {
            let account = stellar_strkey::ed25519::PublicKey::from_string(&from)
                .unwrap()
                .0;
            let sequence = env
                .read_account_from_ledger(account)
                .unwrap()
                .unwrap()
                .seq_num;
            let new_sequence = sequence as i64 + 1;

            let address = Address::from_string(&SString::from_str(&env.soroban(), &from));

            let tx = env
                .simulate_contract_call_to_tx(
                    from,
                    new_sequence,
                    stellar_strkey::Contract::from_string(&contract).unwrap().0,
                    Symbol::new(&env.soroban(), "claim_reward"),
                    vec![&env.soroban(), address.into_val(env.soroban())],
                )
                .unwrap()
                .tx
                .unwrap();

            // access the response objet and change the resources
            let tx = TransactionEnvelope::from_xdr_base64(tx, Limits::none()).unwrap();
            let TransactionEnvelope::Tx(TransactionV1Envelope { mut tx, .. }) = tx else {
                panic!()
            };
            let TransactionExt::V1(mut v1ext) = tx.ext else {
                panic!()
            };
            let mut r = v1ext.resources;
            r.read_bytes += 200;
            r.write_bytes += 100;
            v1ext.resource_fee += 100;
            v1ext.resources = r;
            tx.ext = TransactionExt::V1(v1ext);
            tx.fee += 100;

            // build again the simulated transaction after we updated the resources
            let envelope = TransactionEnvelope::Tx(TransactionV1Envelope {
                tx,
                signatures: std::vec![].try_into().unwrap(),
            });
            Response {
                tx: envelope.to_xdr_base64(Limits::none()).unwrap(),
            }
        }
    };

    env.conclude(response)
}

#[test]
fn test() {
    println!(
        "{:?}",
        stellar_strkey::Contract::from_string(
            "CAYRXB3BP4VPUTH6NFAI543HOVOD6AVN4KKASSIR3GALG23EV5DQT5G5"
        )
        .unwrap()
        .0
    );
}
