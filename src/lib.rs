use serde::Serialize;
use types::*;
use zephyr_sdk::{
    prelude::*,
    soroban_sdk::{self, vec, xdr::{self, ScVal, TransactionEnvelope, TransactionExt, TransactionV1Envelope}, Address, Bytes, BytesN, IntoVal, String as SString, Symbol},
    EnvClient,
};
mod types;

const CONTRACT_ADDRESS: [u8; 32] = [49, 27, 135, 97, 127, 42, 250, 76, 254, 105, 64, 142, 243, 103, 117, 92, 63, 2, 173, 226, 148, 9, 73, 17, 217, 128, 179, 107, 100, 175, 71, 9];

#[no_mangle]
pub extern "C" fn on_close() {
    let env = EnvClient::new();
}            

#[derive(Serialize)]
pub struct Response {
    tx: String
}

#[no_mangle]
pub extern "C" fn simulate() {
    let env = EnvClient::empty();
    let request: SimulationRequest = env.read_request_body();
    //let new_sequence = env.soroban().ledger().sequence() + 1;

    let response = match request {
        SimulationRequest::Deposit(SimulateDeposit { from, amount }) => {
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
            let tx = env.simulate_contract_call_to_tx(
                from,
                new_sequence,
                CONTRACT_ADDRESS,
                Symbol::new(&env.soroban(), "deposit"),
                vec![
                    &env.soroban(),
                    address.into_val(env.soroban()),
                    (amount as i128).into_val(env.soroban()),
                ],
            ).unwrap().tx.unwrap();

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
            let envelope = TransactionEnvelope::Tx(TransactionV1Envelope { tx, signatures: std::vec![].try_into().unwrap() });
            Response {tx: envelope.to_xdr_base64(Limits::none()).unwrap() }
        },
        /*
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
        */
    };

    env.conclude(response)
}


#[test]
fn test() {
    println!("{:?}", stellar_strkey::Contract::from_string("CAYRXB3BP4VPUTH6NFAI543HOVOD6AVN4KKASSIR3GALG23EV5DQT5G5").unwrap().0);   
}