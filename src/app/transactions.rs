use crate::WalletApp;

pub fn page(app_state: &mut WalletApp, ui: &mut egui::Ui) {
    ui.heading("Transactions");
    ui.horizontal(|ui| {
        ui.label("Next receive address: ");
        ui.label(format!("{}", app_state.receive.derivation));
    });
}
