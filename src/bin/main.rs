use bitcoin::network::constants::Network;
use bitcoin_wallet::{
    account::{Account, AccountAddressType, MasterAccount, Unlocker},
    mnemonic::Mnemonic,
};
use grin_btc_poc::{mine_blocks, send_to_address};

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
    let account = Account::new(&mut unlocker, AccountAddressType::P2WPKH, 1, 0, 10).unwrap();
    master.add_account(account);

    let address = master
        .get_mut((1, 0))
        .unwrap()
        .next_key()
        .unwrap()
        .address
        .clone();

    println!("Regtest address: {}", address);

    mine_blocks(1);

    let txid = send_to_address(address);

    println!("Transaction ID: {}", txid.unwrap());
}
