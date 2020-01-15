use bitcoin::{hashes::sha256d::Hash as TxId, Address};
use std::str::FromStr;

pub fn mine_blocks(n: u32) {
    ureq::post("http://user:password@localhost:18443").send_json(
        ureq::json!({"jsonrpc": "1.0", "id":"curltest", "method": "generate", "params": [n] }),
    );
}

pub fn send_to_address(address: Address) -> Result<TxId, ()> {
    let res = ureq::post("http://user:password@localhost:18443")
        .send_json(ureq::json!({"jsonrpc": "1.0", "id":"curltest", "method": "sendtoaddress", "params": [format!("{}", address), 1] }));

    if res.ok() {
        return Ok(TxId::from_str(
            res.into_json()
                .unwrap()
                .as_object()
                .unwrap()
                .get("result")
                .unwrap()
                .as_str()
                .unwrap(),
        )
        .unwrap());
    } else {
        return Err(());
    }
}
