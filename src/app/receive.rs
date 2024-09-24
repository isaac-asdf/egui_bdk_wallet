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

pub fn page(state: &mut WalletApp, ui: &mut egui::Ui) {
    ui.heading("Receive");
    for i in 0..state.receive.next_addr.len() {
        ui.horizontal(|ui| {
            ui.label(format!("Unused Address {:02}:", i + 1));
            ui.label(format!("{}", state.receive.next_addr[i]));
            if ui.button("Mark used").clicked() {
                state.debug.push("mark used unimplemented".into());
                // state
                //     .wallet_req
                //     .send(crate::messages::WalletRequest::MarkUsed(
                //         state.receive.next_addr[i],
                //     ));
            }
        });
    }
}
