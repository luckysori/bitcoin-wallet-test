use bitcoin::{
    blockdata::transaction::TxOut,
    hashes::sha256d::Hash as TxId,
    util::psbt::serialize::{Deserialize, Serialize},
    Address, OutPoint, Transaction,
};
use std::str::FromStr;

pub fn generate_block(n: u32) {
    ureq::post("http://user:password@localhost:18443").send_json(
        ureq::json!({"jsonrpc": "1.0", "id":"grin-btc-poc", "method": "generate", "params": [n] }),
    );
}

pub fn send_to_address(address: Address) -> Result<(TxOut, OutPoint), ()> {
    let res = ureq::post("http://user:password@localhost:18443")
        .send_json(ureq::json!({"jsonrpc": "1.0", "id":"grin-btc-poc", "method": "sendtoaddress", "params": [format!("{}", address), 1] }));

    if res.ok() {
        let txid = TxId::from_str(
            res.into_json()
                .unwrap()
                .as_object()
                .unwrap()
                .get("result")
                .unwrap()
                .as_str()
                .unwrap(),
        )
        .unwrap();

        let raw_tx = get_rawtransaction(txid).unwrap();
        let vout = find_vout(&raw_tx, &address).unwrap();

        let txout = TxOut {
            value: 100_000_000,
            script_pubkey: address.script_pubkey(),
        };
        let outpoint = OutPoint { txid, vout };

        Ok((txout, outpoint))
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

fn find_vout(transaction: &Transaction, to_address: &Address) -> Option<u32> {
    let to_address_script_pubkey = to_address.script_pubkey();

    transaction
        .output
        .iter()
        .enumerate()
        .find(|(_, txout)| txout.script_pubkey == to_address_script_pubkey)
        .map(|(vout, _)| vout as u32)
}
