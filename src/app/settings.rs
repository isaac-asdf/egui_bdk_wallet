use bdk_wallet::keys::bip39::Mnemonic;

use crate::{bdk_utils, messages::WalletRequest, WalletApp};

pub fn page(app_state: &mut WalletApp, ui: &mut egui::Ui) {
    ui.heading("Settings");

    ui.horizontal(|ui| {
        ui.label("Electrum URL: ");
        ui.text_edit_singleline(&mut app_state.settings.electrum_url);
    });

    ui.horizontal(|ui| {
        ui.label("DB URL: ");
        ui.text_edit_singleline(&mut app_state.settings.wallet_db);
        if ui.button("Save wallet").clicked() {
            // app_state.wallet.database().sa
        }
    });

    ui.add_space(10.);
    ui.heading("New wallet");
    ui.text_edit_multiline(&mut app_state.settings.new_wallet_seed);
    if ui.button("Create new wallet").clicked() {
        //
        let new = Mnemonic::parse(&app_state.settings.new_wallet_seed);
        match new {
            Ok(new) => app_state
                .wallet_req
                .send(WalletRequest::CreateNew(bdk_utils::from_words("test", new)))
                .expect("bg failed"),
            Err(_) => app_state.settings.new_wallet_seed += " seed parse failed",
        };
    }
}
