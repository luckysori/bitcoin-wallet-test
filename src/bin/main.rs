use bitcoin::{network::constants::Network, Script, SigHashType, Transaction, TxIn, TxOut};
use bitcoin_wallet::{
    account::{Account, AccountAddressType, MasterAccount, Unlocker},
    coins::Coins,
    mnemonic::Mnemonic,
};
use grin_btc_poc::{generate_block, get_rawtransaction, send_rawtransaction, send_to_address};

fn main() {
    const PASSPHRASE: &str = "correct horse battery staple";
    let words = "announce damage viable ticket engage curious yellow ten clock finish burden orient faculty rigid smile host offer affair suffer slogan mercy another switch park";
    let mnemonic = Mnemonic::from_str(words).unwrap();
    let mut master =
        MasterAccount::from_mnemonic(&mnemonic, 0, Network::Regtest, PASSPHRASE, None).unwrap();

    let mut unlocker = Unlocker::new_for_master(&master, PASSPHRASE).unwrap();

    let sender_account = Account::new(&mut unlocker, AccountAddressType::P2WPKH, 1, 0, 10).unwrap();
    master.add_account(sender_account);
    let sender_address = master
        .get_mut((1, 0))
        .unwrap()
        .next_key()
        .unwrap()
        .address
        .clone();

    let mut coins = Coins::new();

    // fund bitcoind wallet
    generate_block(1, &mut master, &mut coins);

    // fund sender address
    send_to_address(sender_address).expect("could not fund sender address");

    let receiver_account =
        Account::new(&mut unlocker, AccountAddressType::P2WPKH, 2, 0, 10).unwrap();
    master.add_account(receiver_account);
    let receiver_address = master
        .get_mut((2, 0))
        .unwrap()
        .next_key()
        .unwrap()
        .address
        .clone();
    dbg!(receiver_address.clone());

    let change_address = master
        .get_mut((1, 0))
        .expect("couldn't get mut")
        .next_key()
        .expect("no next key")
        .address
        .clone();

    // we believe it should not work with 1 and |_| Some(10). Investigate
    let inputs = coins.choose_inputs(100_000_000, 1, |_| Some(10));

    let mut spending_transaction = Transaction {
        input: vec![TxIn {
            previous_output: inputs[0].0,
            sequence: 0xffffffff,
            witness: Vec::new(),
            script_sig: Script::new(),
        }],
        output: vec![
            TxOut {
                script_pubkey: receiver_address.script_pubkey(),
                value: 50_000_000,
            },
            TxOut {
                script_pubkey: change_address.script_pubkey(),
                value: 50_000_000 - 1_000,
            },
        ],
        lock_time: 0,
        version: 2,
    };

    master
        .sign(
            &mut spending_transaction,
            SigHashType::All,
            &(|_| Some(inputs[0].1.output.clone())),
            &mut unlocker,
        )
        .expect("can not sign");

    send_rawtransaction(spending_transaction.clone()).unwrap();

    dbg!(get_rawtransaction(spending_transaction.txid()).unwrap());
}
