use std::{io::Write, str::FromStr};

use crate::WalletApp;
use bdk_wallet::{
    bitcoin::{Address, Network, Psbt},
    LocalOutput,
};

#[derive(Clone, Debug)]
pub struct SendState {
    pub pay_to_addr: Option<Address>,
    addr_entry: String,
    pub label: String,
    pub sats_amount: u64,
    pub sats_entry: String,
    pub selected_utxos: Vec<LocalOutput>,
    pub fee_rate: f32,
    pub fees: u64,
    pub psbt: Option<Psbt>,
}

impl SendState {
    pub fn new() -> Self {
        SendState {
            pay_to_addr: None,
            addr_entry: "".into(),
            label: "".into(),
            sats_amount: 0,
            sats_entry: "".into(),
            selected_utxos: Vec::new(),
            fee_rate: 1.,
            fees: 0,
            psbt: None,
        }
    }

    // fn is_signed(self) -> bool {
    //     if let Some(psbt) = self.psbt {
    //         // psbt.finalize()
    //         true
    //     } else {
    //         false
    //     }
    // }

    fn get_psbt(&self) -> Option<Vec<u8>> {
        match self.psbt.clone() {
            Some(psbt) => Some(psbt.serialize()),
            None => None,
        }
    }

    fn verify_address(&mut self, network: Network) {
        let addr = Address::from_str(&self.addr_entry);
        if let Ok(a) = addr {
            let res = a.require_network(network);
            if let Ok(res) = res {
                self.pay_to_addr = Some(res);
            } else {
                self.addr_entry = "Invalid address".into();
            }
        } else {
            self.addr_entry = addr.err().unwrap().to_string();
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
        ui.label("Destination Address:");
        ui.text_edit_singleline(&mut app_state.send.addr_entry);

        if ui.button("Verify address").clicked() {
            app_state.send.verify_address(app_state.network);
        }

        if app_state.send.pay_to_addr.is_some() {
            ui.label("Address verified");
        }
    });

    ui.horizontal(|ui| {
        ui.label("Amount (sats): ");
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

    if app_state.send.psbt.is_some() {
        let psbt = app_state.send.psbt.as_mut().unwrap().clone();

        if ui.button("Download PSBT").clicked() {
            let mut dl = dirs::download_dir().unwrap();
            dl.push("psbt.txt");
            if let Ok(mut f) = std::fs::File::create(dl) {
                if let Some(vec) = app_state.send.get_psbt() {
                    f.write(&vec).unwrap();
                }
            }
        }

        if ui.button("Broadcast Transaction").clicked() {
            app_state
                .wallet_req
                .send(crate::messages::WalletRequest::SendTransaction(psbt))
                .unwrap()
        }
    }
}
