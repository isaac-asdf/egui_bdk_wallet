use crate::{messages::WalletRequest, WalletApp};

use bdk_wallet::{keys::bip39::Mnemonic, KeychainKind, Wallet};

pub fn page(app_state: &mut WalletApp, ui: &mut egui::Ui) {
    ui.heading("Home");

    if ui.button("Sync").clicked() {
        app_state
            .wallet_req
            .send(WalletRequest::Sync)
            .expect("bg failed");
    }

    ui.label(format!("{:?}", app_state.home.balance));
}
