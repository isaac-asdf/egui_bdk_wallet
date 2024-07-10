use crate::{messages::WalletRequest, WalletApp};

use bdk_wallet::{keys::bip39::Mnemonic, KeychainKind, Wallet};

const DB_PATH: &str = "bdk-from-sparrow";

pub fn page(app_state: &mut WalletApp, ui: &mut egui::Ui) {
    ui.heading("Home");

    if ui.button("Sync").clicked() {
        app_state
            .wallet_req
            .send(WalletRequest::Sync(app_state.settings.wallet_db.clone()));
    }

    ui.label(format!("{:?}", app_state.home.balance));
}
