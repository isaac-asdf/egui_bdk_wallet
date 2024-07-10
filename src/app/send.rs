use crate::WalletApp;

pub fn page(app_state: &mut WalletApp, ui: &mut egui::Ui) {
    ui.heading("Transaction Builder");
    ui.horizontal(|ui| {
        ui.label("Label: ");
        ui.label(format!("{}", app_state.send.label));
    });
}
