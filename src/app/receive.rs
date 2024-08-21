use crate::WalletApp;
use bdk_wallet::AddressInfo;

pub struct ReceiveState {
    pub pay_to_addr: String,
    pub label: String,
    pub derivation: String,
    pub next_addr: Vec<AddressInfo>,
}

impl ReceiveState {
    pub fn new() -> Self {
        ReceiveState {
            pay_to_addr: "".into(),
            label: "".into(),
            derivation: "".into(),
            next_addr: Vec::new(),
        }
    }
}

pub fn page(app_state: &mut WalletApp, ui: &mut egui::Ui) {
    ui.heading("Receive");
    ui.horizontal(|ui| {
        ui.label("Next receive address: ");
        if app_state.receive.next_addr.len() > 0 {
            ui.label(format!("{}", app_state.receive.next_addr[0]));
        }
    });
}
