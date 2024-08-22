use types::*;
use zephyr_sdk::{
    prelude::*,
    soroban_sdk::{vec, xdr::ScVal, Address, Bytes, BytesN, IntoVal, String as SString, Symbol},
    EnvClient,
};

mod types;
#[no_mangle]
pub extern "C" fn on_close() {
    let env = EnvClient::new();
}            

#[no_mangle]
pub extern "C" fn simulate() {
    let env = EnvClient::empty();
    let request: SimulationRequest = env.read_request_body();

    let response = match request {
        SimulationRequest::Deposit(SimulateDeposit { from, sequence, amount }) => {
            let address = Address::from_string(&SString::from_str(&env.soroban(), &from));
            // amount is i64: good like this? 
            env.simulate_contract_call_to_tx(
                from,
                sequence,
                CONTRACT_ADDRESS,
                Symbol::new(&env.soroban(), "deposit"),
                vec![
                    &env.soroban(),
                    address.into_val(env.soroban()),
                    amount.into_val(env.soroban()),
                ],
            )
            
        },
        SimulationRequest::Vote(SimulateVote { from, sequence, hash, upvote }) => {
            let address = Address::from_string(&SString::from_str(&env.soroban(), &from));
            let hash = BytesN::<32>::from_array(&env.soroban(), &to_array::<u8, 32>(hex::decode(hash).unwrap()));
            let action = if upvote {
                "upvote"
            } else {
                "downvote"
            };

            env.simulate_contract_call_to_tx(
                from,
                sequence,
                CONTRACT_ADDRESS,
                Symbol::new(&env.soroban(), action),
                vec![
                    &env.soroban(),
                    address.into_val(env.soroban()),
                    hash.into_val(env.soroban())
                ],
            )
        }
    }
    .unwrap();

    env.conclude(response)
}