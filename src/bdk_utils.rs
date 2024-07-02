use bdk_wallet::{
    bitcoin::Network,
    keys::{bip39::Mnemonic, DerivableKey, ExtendedKey},
    template::Bip84,
    wallet::{ChangeSet, Wallet},
    KeychainKind,
};

use bdk_electrum::electrum_client;
use bdk_electrum::BdkElectrumClient;
use bdk_file_store::Store;
use std::collections::HashSet;
use std::io::Write;

use crate::WalletApp;

const STOP_GAP: usize = 50;
const BATCH_SIZE: usize = 5;

pub fn create_new() -> Wallet {
    // Open or create a new file store for wallet data.
    let db_path = std::env::temp_dir().join("bdk-electrum-example-new");
    let mut db = bdk_file_store::Store::<ChangeSet>::open_or_create_new(b"magic_bytes", db_path)
        .expect("create store");
    let changeset = db.aggregate_changesets().expect("changeset loaded");

    // Create a wallet with initial wallet data read from the file store.
    let descriptor = "wpkh(tprv8ZgxMBicQKsPdcAqYBpzAFwU5yxBUo88ggoBqu1qPcHUfSbKK1sKMLmC7EAk438btHQrSdu3jGGQa6PA71nvH5nkDexhLteJqkM4dQmWF9g/84'/1'/0'/0/*)";
    let change_descriptor = "wpkh(tprv8ZgxMBicQKsPdcAqYBpzAFwU5yxBUo88ggoBqu1qPcHUfSbKK1sKMLmC7EAk438btHQrSdu3jGGQa6PA71nvH5nkDexhLteJqkM4dQmWF9g/84'/1'/0'/1/*)";
    let wallet = Wallet::new_or_load(descriptor, change_descriptor, changeset, Network::Testnet)
        .expect("create or load wallet");
    wallet
}

pub fn from_words(words: Mnemonic) -> Wallet {
    let db_path = std::env::temp_dir().join("bdk-from-sparrow");
    let mut db = bdk_file_store::Store::<ChangeSet>::open_or_create_new(b"magic_bytes", db_path)
        .expect("create store");
    let changeset = db.aggregate_changesets().expect("changeset loaded");

    // create main desc
    let xkey: ExtendedKey = words.into_extended_key().unwrap();
    let xprv = xkey.into_xprv(Network::Testnet).unwrap();
    let wallet = Wallet::new_or_load(
        Bip84(xprv.clone(), KeychainKind::External),
        Bip84(xprv, KeychainKind::Internal),
        changeset,
        Network::Testnet,
    )
    .unwrap();
    wallet
}

pub fn sync_db(app_state: &mut WalletApp) {
    let db_path = std::env::temp_dir().join("bdk-from-sparrow");
    let mut db = bdk_file_store::Store::<bdk_wallet::wallet::ChangeSet>::open_or_create_new(
        b"magic_bytes",
        db_path,
    )
    .unwrap();

    let client = BdkElectrumClient::new(
        electrum_client::Client::new("ssl://electrum.blockstream.info:60002").unwrap(),
    );

    // Populate the electrum client's transaction cache so it doesn't redownload transaction we
    // already have.
    client.populate_tx_cache(&app_state.wallet);

    let request = app_state
        .wallet
        .start_full_scan()
        .inspect_spks_for_all_keychains({
            let mut once = HashSet::<KeychainKind>::new();
            move |k, spk_i, _| {
                if once.insert(k) {
                    print!("\nScanning keychain [{:?}]", k)
                } else {
                    print!(" {:<3}", spk_i)
                }
            }
        })
        .inspect_spks_for_all_keychains(|_, _, _| std::io::stdout().flush().expect("must flush"));
    let mut update = client
        .full_scan(request, STOP_GAP, BATCH_SIZE, false)
        .unwrap()
        .with_confirmation_time_height_anchor(&client)
        .unwrap();

    let now = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs();
    let _ = update.graph_update.update_last_seen_unconfirmed(now);

    app_state.wallet.apply_update(update).unwrap();
    if let Some(changeset) = app_state.wallet.take_staged() {
        db.append_changeset(&changeset).unwrap();
    }

    let balance = app_state.wallet.balance();
    app_state.debug = format!("Wallet balance after syncing: {} sats", balance.total());
}
