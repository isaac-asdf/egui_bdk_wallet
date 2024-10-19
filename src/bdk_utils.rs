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
use std::{
    io::{LineWriter, Read, Write},
    path::PathBuf,
};

const STOP_GAP: usize = 50;
const BATCH_SIZE: usize = 5;

const NETWORK: Network = Network::Testnet;

pub fn broadcast_tx(tx: &Transaction, elec_url: &str) -> Result<Txid, String> {
    let client = BdkElectrumClient::new(electrum_client::Client::new(elec_url).unwrap());

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
        .filter(|f| !f.contains("keys"))
        .collect()
}

pub fn from_changeset(db_path: &str, name: &str) -> Result<PersistedWallet<Connection>, bool> {
    let mut path = PathBuf::from(db_path);
    path.push(name);
    let mut db = Connection::open(&path).unwrap();
    let kpath = String::from(name) + "_keys";
    path.set_file_name(kpath);
    let wallet = if let Ok(mut f) = std::fs::File::open(path) {
        let wallet = Wallet::load().extract_keys();
        let mut tprv = String::new();
        f.read_to_string(&mut tprv).unwrap();
        let mut keys = tprv.lines();
        let extkey: String = keys.next().unwrap().to_owned();
        let intkey: String = keys.next().unwrap().to_owned();
        wallet
            .descriptor(KeychainKind::Internal, Some(intkey))
            .descriptor(KeychainKind::External, Some(extkey))
    } else {
        Wallet::load()
    };
    let wallet = wallet.load_wallet(&mut db);
    match wallet {
        Ok(w) => match w {
            Some(w) => Ok(w),
            None => Err(false),
        },
        Err(e) => {
            println!("{:?}", e);
            Err(false)
        }
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

pub fn from_words(
    db_path: &str,
    name: &str,
    words: Mnemonic,
    save_seed: bool,
) -> PersistedWallet<Connection> {
    let mut path = PathBuf::from(db_path);
    path.push(name);
    let mut db = Connection::open(&path).unwrap();
    let xkey: ExtendedKey = words.into_extended_key().unwrap();
    let xprv = xkey.into_xprv(NETWORK).unwrap();
    let wallet = Wallet::create(
        Bip84(xprv.clone(), KeychainKind::External),
        Bip84(xprv.clone(), KeychainKind::Internal),
    )
    .network(NETWORK)
    .create_wallet(&mut db)
    .unwrap();

    if save_seed {
        let keys_name = String::from(name) + "_keys";
        path.set_file_name(keys_name);

        let f = std::fs::File::create(path).unwrap();
        let mut lr = LineWriter::new(f);

        let mut bs = Vec::new();
        for ele in wallet.get_signers(KeychainKind::External).signers() {
            let test = ele.descriptor_secret_key().unwrap().to_string();
            bs.push(format!("wpkh({test})\n"));
        }
        for ele in wallet.get_signers(KeychainKind::Internal).signers() {
            let test = ele.descriptor_secret_key().unwrap().to_string();
            bs.push(format!("wpkh({test})\n"));
        }

        bs.iter().for_each(|key| {
            lr.write(key.as_bytes()).unwrap();
        });
    }

    wallet
}

pub fn cp_sync(
    db_path: &str,
    name: &str,
    wallet: &mut PersistedWallet<Connection>,
    elec_url: &str,
) -> Balance {
    let client = BdkElectrumClient::new(electrum_client::Client::new(elec_url).unwrap());
    let sync_request = wallet.start_sync_with_revealed_spks().build();
    let update = client.sync(sync_request, BATCH_SIZE, true).unwrap();

    // Apply the update to the wallet
    wallet.apply_update(update).unwrap();
    persist(db_path, name, wallet);
    let balance = wallet.balance();
    balance
}

pub fn full_scan(
    db_path: &str,
    name: &str,
    wallet: &mut PersistedWallet<Connection>,
    elec_url: &str,
) -> Balance {
    let client = BdkElectrumClient::new(electrum_client::Client::new(elec_url).unwrap());

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet() {
        let words = "section attitude true fabric foam ribbon chaos cradle ordinary venture fat ensure winter skate error glove pulse dolphin they cable verify wolf rain ribbon";
        let mne = Mnemonic::parse(words).unwrap();

        let mut p = PathBuf::from("./tests");
        p.push("tw");
        let _ = std::fs::remove_file(p);
        from_words("./tests/", "tw", mne, true);
        assert!(true);
    }

    #[test]
    fn from_tprv() {
        let w = from_changeset("./tests/", "tw").unwrap();
        w.keychains().for_each(|kc| {
            println!("keychain:{:?}", kc);
        });
    }
}
