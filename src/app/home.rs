use crate::{bdk_utils, WalletApp};

use bdk::blockchain::ElectrumBlockchain;
use bdk::electrum_client::Client;
use bdk::keys::bip39::Mnemonic;
use bdk::{KeychainKind, SyncOptions};

pub fn home(app_state: &mut WalletApp, ui: &mut egui::Ui) {
    ui.heading("Home");

    if ui.button("Create Wallet").clicked() {
        // Parse a mnemonic
        let words = Mnemonic::parse(&app_state.wallet_words);
        if let Ok(words) = words {
            app_state.wallet = bdk_utils::from_words(words);
        } else {
            app_state.wallet_words += " word parse failed";
        }
    }

    if ui.button("Sync").clicked() {
        let client = Client::new(&app_state.electrum_url).unwrap();
        let blockchain = ElectrumBlockchain::from(client);
        let res = app_state.wallet.sync(&blockchain, SyncOptions::default());
        if res.is_err() {
            app_state.debug = res.err().unwrap().to_string();
        } else {
            let first = app_state.wallet.list_transactions(false).unwrap();
            let first = first.first();
            app_state.debug = format!("{:?}", first);
        }
    }

    if app_state.debug.len() > 0 {
        ui.label("Debug");
        ui.text_edit_multiline(&mut app_state.debug);
    }

    ui.label("Words");
    ui.text_edit_multiline(&mut app_state.wallet_words);
    ui.label("Wallet Info");
    ui.label(format!(
        "{:#?}",
        app_state
            .wallet
            .get_descriptor_for_keychain(KeychainKind::External)
            .to_string()
    ));
}
