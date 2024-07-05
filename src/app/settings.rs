use crate::WalletApp;

pub fn settings(app_state: &mut WalletApp, ui: &mut egui::Ui) {
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

    ui.label(&app_state.debug);
}
