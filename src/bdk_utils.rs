use bdk_wallet::{
    bitcoin::{
        key::rand::{thread_rng, Rng},
        Network,
    },
    chain::Persisted,
    keys::{bip39::Mnemonic, DerivableKey, ExtendedKey},
    template::Bip84,
    Balance, KeychainKind, PersistedWallet, Wallet,
};

use bdk_electrum::electrum_client;
use bdk_electrum::BdkElectrumClient;
use bdk_wallet::rusqlite::Connection;
use std::path::PathBuf;

const DB_PATH: &str = "/home/isaac/Desktop/wallets/";

const STOP_GAP: usize = 50;
const BATCH_SIZE: usize = 5;

pub fn list_wallets() -> Vec<String> {
    let files = std::fs::read_dir(DB_PATH).unwrap();
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

pub fn from_changeset(db: &str) -> Result<Persisted<Wallet>, bool> {
    let path = String::from(DB_PATH) + db;
    let mut db = Connection::open(PathBuf::from(path)).unwrap();
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

pub fn from_words(name: &str, words: Mnemonic) -> PersistedWallet {
    let path = String::from(DB_PATH) + name;
    let mut db = Connection::open(PathBuf::from(path)).unwrap();
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

pub fn cp_sync(name: &str, wallet: &mut PersistedWallet) -> Balance {
    let client = BdkElectrumClient::new(
        electrum_client::Client::new("ssl://electrum.blockstream.info:60002").unwrap(),
    );
    let sync_request = wallet.start_sync_with_revealed_spks();
    let update = client.sync(sync_request, BATCH_SIZE, true).unwrap();

    // Apply the update to the wallet
    wallet.apply_update(update).unwrap();
    persist(name, wallet);
    let balance = wallet.balance();
    balance
}

pub fn full_scan(name: &str, wallet: &mut PersistedWallet) -> Balance {
    let client = BdkElectrumClient::new(
        electrum_client::Client::new("ssl://electrum.blockstream.info:60002").unwrap(),
    );

    // Perform the initial full scan on the wallet
    let full_scan_request = wallet.start_full_scan();
    let mut update = client
        .full_scan(full_scan_request, STOP_GAP, BATCH_SIZE, true)
        .unwrap();

    let now = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs();
    let _ = update.graph_update.update_last_seen_unconfirmed(now);

    wallet.apply_update(update).unwrap();
    persist(name, wallet);
    let balance = wallet.balance();
    balance
}

fn persist(name: &str, wallet: &mut PersistedWallet) {
    let path = String::from(DB_PATH) + name;
    let mut db = Connection::open(PathBuf::from(path)).unwrap();
    wallet.persist(&mut db).expect("persist error");
}
