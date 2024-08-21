use crate::WalletApp;

#[derive(Debug, Clone)]
pub struct Settings {
    pub electrum_url: String,
    pub wallet_db: String,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            electrum_url: "ssl://electrum.blockstream.info:60002".into(),
            wallet_db: "wallets".into(),
        }
    }
}

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
}
