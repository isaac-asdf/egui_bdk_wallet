use crate::WalletApp;

pub fn page(app_state: &mut WalletApp, ui: &mut egui::Ui) {
    ui.heading("Receive");
    ui.horizontal(|ui| {
        ui.label("Next receive address: ");
        if app_state.receive.next_addr.len() > 0 {
            ui.label(format!("{}", app_state.receive.next_addr[0]));
        }
    });
}
