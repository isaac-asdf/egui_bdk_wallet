use bdk_wallet::{
    bitcoin::{
        key::rand::{thread_rng, Rng},
        Network, Transaction, Txid,
    },
    keys::{bip39::Mnemonic, DerivableKey, ExtendedKey},
    template::Bip84,
    Balance, KeychainKind, PersistedWallet, Wallet,
};

use bdk_electrum::electrum_client;
use bdk_electrum::BdkElectrumClient;
use bdk_wallet::rusqlite::Connection;
use std::path::PathBuf;

const STOP_GAP: usize = 50;
const BATCH_SIZE: usize = 5;

pub fn broadcast_tx(tx: &Transaction) -> Result<Txid, String> {
    let client = BdkElectrumClient::new(
        electrum_client::Client::new("ssl://electrum.blockstream.info:60002").unwrap(),
    );

    client.transaction_broadcast(tx).map_err(|e| e.to_string())
}

pub fn list_wallets(db_path: &str) -> Vec<String> {
    let files = std::fs::read_dir(db_path).unwrap();
    files
        .into_iter()
        .filter_map(|f| {
            if let Ok(entry) = f {
                return Some(entry.file_name().into_string().unwrap());
            };
            None
        })
        .collect()
}

pub fn from_changeset(db_path: &str, name: &str) -> Result<PersistedWallet<Connection>, bool> {
    let mut path = PathBuf::from(db_path);
    path.push(name);
    let mut db = Connection::open(&path).unwrap();
    let wallet = Wallet::load().load_wallet(&mut db);
    match wallet {
        Ok(w) => match w {
            Some(w) => Ok(w),
            None => Err(false),
        },
        Err(_) => Err(false),
    }
}

/// Create a wallet that is persisted to SQLite database.
pub fn new_seed() -> Mnemonic {
    // Create a new random number generator
    let mut rng = thread_rng();
    // Generate 256 bits of entropy
    let entropy: [u8; 32] = rng.gen();
    let words = Mnemonic::from_entropy(&entropy).unwrap();
    // Create extended key to generate descriptors
    words
}

pub fn from_words(db_path: &str, name: &str, words: Mnemonic) -> PersistedWallet<Connection> {
    let mut path = PathBuf::from(db_path);
    path.push(name);
    let mut db = Connection::open(&path).unwrap();
    let xkey: ExtendedKey = words.into_extended_key().unwrap();
    let xprv = xkey.into_xprv(Network::Testnet).unwrap();
    let wallet = Wallet::create(
        Bip84(xprv.clone(), KeychainKind::External),
        Bip84(xprv, KeychainKind::Internal),
    )
    .network(Network::Testnet)
    .create_wallet(&mut db)
    .unwrap();

    wallet
}

pub fn cp_sync(db_path: &str, name: &str, wallet: &mut PersistedWallet<Connection>) -> Balance {
    let client = BdkElectrumClient::new(
        electrum_client::Client::new("ssl://electrum.blockstream.info:60002").unwrap(),
    );
    let sync_request = wallet.start_sync_with_revealed_spks().build();
    let update = client.sync(sync_request, BATCH_SIZE, true).unwrap();

    // Apply the update to the wallet
    wallet.apply_update(update).unwrap();
    persist(db_path, name, wallet);
    let balance = wallet.balance();
    balance
}

pub fn full_scan(db_path: &str, name: &str, wallet: &mut PersistedWallet<Connection>) -> Balance {
    let client = BdkElectrumClient::new(
        electrum_client::Client::new("ssl://electrum.blockstream.info:60002").unwrap(),
    );

    // Perform the initial full scan on the wallet
    let full_scan_request = wallet.start_full_scan().build();
    let update = client
        .full_scan(full_scan_request, STOP_GAP, BATCH_SIZE, true)
        .unwrap();

    wallet.apply_update(update).unwrap();
    persist(db_path, name, wallet);
    let balance = wallet.balance();
    balance
}

pub fn persist(db_path: &str, name: &str, wallet: &mut PersistedWallet<Connection>) {
    let mut path = PathBuf::from(db_path);
    path.push(name);
    let mut db = Connection::open(&path).unwrap();
    wallet.persist(&mut db).expect("persist error");
}
