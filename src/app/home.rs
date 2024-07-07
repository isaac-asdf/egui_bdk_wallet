use std::sync::{Arc, Mutex};

use crate::{
    bdk_utils,
    messages::{Sync, WalletRequest, WalletResponse},
    WalletApp,
};

use bdk_wallet::{keys::bip39::Mnemonic, KeychainKind, Wallet};

const DB_PATH: &str = "bdk-from-sparrow";

pub fn home(app_state: &mut WalletApp, ui: &mut egui::Ui) {
    ui.heading("Home");

    let new_wallet = app_state.wallet_updates.try_recv();
    if let Ok(new_wallet) = new_wallet {
        match new_wallet {
            WalletResponse::Echo(e) => app_state.counter = e,
            WalletResponse::Sync(s) => app_state.debug = format!("balance: {}", s),
        };
    }
    ui.horizontal(|ui| {
        ui.label(format!("Last Echo: {}", app_state.counter));
        if ui.button("Send Echo").clicked() {
            app_state
                .wallet_req
                .send(WalletRequest::Echo(app_state.counter + 1))
                .expect("Background thread died");
        }
    });

    if ui.button("Create Wallet from Words").clicked() {
        // Parse a mnemonic
        let words = Mnemonic::parse(&app_state.wallet_info.wallet_words);
        if let Ok(words) = words {
            app_state.wallet = Arc::new(Mutex::new(bdk_utils::from_words(words)));
        } else {
            app_state.wallet_info.wallet_words += " word parse failed";
        }
        // let balance = app_state.wallet.balance();
        // app_state.debug = format!("Wallet balance after syncing: {} sats", balance.total());
    }
    if ui.button("Load changeset").clicked() {
        let db_path = std::env::temp_dir().join(DB_PATH);
        let mut db =
            bdk_file_store::Store::<bdk_wallet::wallet::ChangeSet>::open(b"magic_bytes", db_path)
                .unwrap();
        let changeset = db
            .aggregate_changesets()
            .expect("there must be an existing changeset");
        let wallet = Wallet::load_from_changeset(changeset.unwrap()).unwrap();
        let balance = wallet.balance();
        app_state.debug = format!("Wallet balance after syncing: {} sats", balance.total());
    }

    if ui.button("Sync").clicked() {
        app_state
            .wallet_req
            .send(WalletRequest::Sync(Sync {
                wallet: app_state.wallet.clone(),
                db_path: DB_PATH.into(),
            }))
            .expect("bg thread died");
    }

    if app_state.debug.len() > 0 {
        ui.label("Debug");
        ui.text_edit_multiline(&mut app_state.debug);
    }

    ui.label("Words");
    ui.text_edit_multiline(&mut app_state.wallet_info.wallet_words);
    ui.label("Wallet Info");
    if let Ok(wallet) = app_state.wallet.try_lock() {
        ui.label(format!(
            "{:#?}",
            wallet
                .get_descriptor_for_keychain(KeychainKind::External)
                .to_string()
        ));
    }
}
