use bitcoin::{network::constants::Network, Script, SigHashType, Transaction, TxIn, TxOut};
use bitcoin_wallet::{
    account::{Account, AccountAddressType, MasterAccount, Unlocker},
    mnemonic::Mnemonic,
};
use grin_btc_poc::{generate_block, get_rawtransaction, send_rawtransaction, send_to_address};

fn main() {
    const PASSPHRASE: &str = "correct horse battery staple";

    // re-create a master from a known mnemonic
    let words = "announce damage viable ticket engage curious yellow ten clock finish burden orient faculty rigid smile host offer affair suffer slogan mercy another switch park";
    let mnemonic = Mnemonic::from_str(words).unwrap();
    // PASSPHRASE is used to encrypt the seed in memory and in storage
    // last argument is option password for plausible deniability
    let mut master =
        MasterAccount::from_mnemonic(&mnemonic, 0, Network::Regtest, PASSPHRASE, None).unwrap();

    // The master accounts only store public keys
    // Private keys are created on-demand from encrypted seed with an Unlocker and
    // forgotten as soon as possible

    // create an unlocker that is able to decrypt the encrypted mnemonic and then
    // calculate private keys
    let mut unlocker = Unlocker::new_for_master(&master, PASSPHRASE).unwrap();

    // The unlocker is needed to create accounts within the master account as
    // key derivation follows BIP 44, which requires private key derivation

    // create a P2WPKH (pay-to-witness-public-key-hash) (native single key segwit)
    // account. account number 1, sub-account 0 (which usually means receiver)
    // BIP32 look-ahead 10
    let sender_account = Account::new(&mut unlocker, AccountAddressType::P2WPKH, 1, 0, 10).unwrap();
    master.add_account(sender_account);
    let sender_address = master
        .get_mut((1, 0))
        .unwrap()
        .next_key()
        .unwrap()
        .address
        .clone();

    generate_block(1);

    let (txout, outpoint) = send_to_address(sender_address).unwrap();

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

    let change_account = Account::new(&mut unlocker, AccountAddressType::P2WPKH, 3, 0, 10).unwrap();
    master.add_account(change_account);
    let change_address = master
        .get_mut((3, 0))
        .expect("couldn't get mut")
        .next_key()
        .expect("no next key")
        .address
        .clone();

    let mut spending_transaction = Transaction {
        input: vec![TxIn {
            previous_output: outpoint,
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
            &(|_| Some(txout.clone())),
            &mut unlocker,
        )
        .expect("can not sign");

    send_rawtransaction(spending_transaction.clone()).unwrap();

    dbg!(get_rawtransaction(spending_transaction.txid()).unwrap());
}
