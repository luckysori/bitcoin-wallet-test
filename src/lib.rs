use bitcoin::{
    consensus::deserialize,
    hashes::sha256d::Hash as TxId,
    util::psbt::serialize::{Deserialize, Serialize},
    Address, Transaction,
};
use bitcoin_wallet::{account::MasterAccount, coins::Coins};

// TODO: Make it so that spendable coins can be added for more than one account
// at a time, by for example passing a`Vec<(MasterAccount, Coins)>`.
pub fn generate_block(n: u32, master_account: &mut MasterAccount, coin_store: &mut Coins) {
    let json_res = ureq::post("http://user:password@localhost:18443").send_json(
        ureq::json!({"jsonrpc": "1.0", "id":"grin-btc-poc", "method": "generate", "params": [n] }),
    ).into_json().unwrap();

    let blockhashes = json_res
        .as_object()
        .unwrap()
        .get("result")
        .unwrap()
        .as_array()
        .unwrap();

    for blockhash in blockhashes {
        let res = ureq::post("http://user:password@localhost:18443").send_json(
        ureq::json!({"jsonrpc": "1.0", "id":"grin-btc-poc", "method": "getblock", "params": [blockhash, 0] }),
        );

        let json = res.into_json().unwrap();
        let hex = hex::decode(
            json.as_object()
                .unwrap()
                .get("result")
                .unwrap()
                .as_str()
                .unwrap(),
        )
        .unwrap();

        let block = deserialize(&hex).unwrap();

        coin_store.process(master_account, &block);
    }
}

pub fn send_to_address(address: Address) -> Result<(), ()> {
    let res = ureq::post("http://user:password@localhost:18443")
        .send_json(ureq::json!({"jsonrpc": "1.0", "id":"grin-btc-poc", "method": "sendtoaddress", "params": [format!("{}", address), 1] }));

    if res.ok() {
        Ok(())
    } else {
        Err(())
    }
}

pub fn send_rawtransaction(transaction: Transaction) -> Result<(), ()> {
    let raw_tx = transaction.serialize();

    let res = ureq::post("http://user:password@localhost:18443")
        .send_json(ureq::json!({"jsonrpc": "1.0", "id":"grin-btc-poc", "method": "sendrawtransaction", "params": [hex::encode(raw_tx)] }));

    if res.ok() {
        Ok(())
    } else {
        Err(())
    }
}

pub fn get_rawtransaction(txid: TxId) -> Result<Transaction, ()> {
    let res = ureq::post("http://user:password@localhost:18443")
        .send_json(ureq::json!({"jsonrpc": "1.0", "id":"grin-btc-poc", "method": "getrawtransaction", "params": [format!("{}", txid), 1] }));

    if res.ok() {
        let json = res.into_json().unwrap();
        let hex_tx = json
            .as_object()
            .unwrap()
            .get("result")
            .unwrap()
            .get("hex")
            .unwrap()
            .as_str()
            .unwrap();

        Ok(Transaction::deserialize(&hex::decode(hex_tx).unwrap()).unwrap())
    } else {
        Err(())
    }
}
