use bdk_wallet::{
    bitcoin::Network,
    keys::{bip39::Mnemonic, DerivableKey, ExtendedKey},
    template::Bip84,
    wallet::{ChangeSet, Wallet},
    KeychainKind,
};

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
    let db_path = std::env::temp_dir().join("bdk-electrum-example");
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
