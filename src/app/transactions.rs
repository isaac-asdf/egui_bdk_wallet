use crate::WalletApp;

pub fn transaction(app_state: &mut WalletApp, ui: &mut egui::Ui) {
    ui.heading("Transaction Builder");
    ui.horizontal(|ui| {
        ui.label("Next receive address: ");
        if let Ok(mut wallet) = app_state.wallet.try_lock() {
            ui.label(format!(
                "{}",
                wallet.next_unused_address(bdk_wallet::KeychainKind::External),
            ));
        }
    });
}
