use crate::WalletApp;
use bdk_wallet::LocalOutput;

#[derive(Clone, Debug)]
pub struct SendState {
    pub pay_to_addr: String,
    pub label: String,
    pub sats_amount: u64,
    pub sats_entry: String,
    pub selected_utxos: Vec<LocalOutput>,
    pub fee_rate: f32,
    pub fees: u64,
}

impl SendState {
    pub fn new() -> Self {
        SendState {
            pay_to_addr: "".into(),
            label: "".into(),
            sats_amount: 0,
            sats_entry: "".into(),
            selected_utxos: Vec::new(),
            fee_rate: 1.,
            fees: 0,
        }
    }
}

pub fn page(app_state: &mut WalletApp, ui: &mut egui::Ui) {
    ui.heading("Transaction Builder");
    ui.horizontal(|ui| {
        ui.label("Label: ");
        ui.label(format!("{}", app_state.send.label));
    });

    ui.horizontal(|ui| {
        ui.label("Amount: ");
        if ui
            .text_edit_singleline(&mut app_state.send.sats_entry)
            .changed()
        {
            match app_state.send.sats_entry.parse::<u64>() {
                Ok(val) => app_state.send.sats_amount = val,
                Err(_) => {
                    app_state.send.sats_amount = 0;
                    app_state.send.sats_entry = app_state
                        .send
                        .sats_entry
                        .chars()
                        .filter(|c| c.is_digit(10))
                        .collect();
                }
            };
        }
    });

    ui.horizontal(|ui| {
        ui.label("UTXOS:");
        app_state.send.selected_utxos.iter().for_each(|utxo| {
            ui.label(format!("{} sats", utxo.txout.value));
        });
    });

    if ui.button("Create TX").clicked() {
        app_state
            .wallet_req
            .send(crate::messages::WalletRequest::CreateTransaction(
                app_state.send.clone().into(),
            ))
            .unwrap();
    }
}
