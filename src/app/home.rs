use crate::{messages::WalletRequest, WalletApp};
use bdk_wallet::bitcoin::Transaction;
use bdk_wallet::Balance;

#[derive(Debug, Clone)]
pub struct HomeState {
    pub balance: Option<Balance>,
    transactions: Vec<Transaction>,
}

impl HomeState {
    pub fn new() -> Self {
        HomeState {
            balance: None,
            transactions: Vec::new(),
        }
    }
}

pub fn page(app_state: &mut WalletApp, ui: &mut egui::Ui) {
    ui.heading("Home");

    if ui.button("Sync").clicked() {
        app_state
            .wallet_req
            .send(WalletRequest::Sync)
            .expect("bg failed");
    }

    ui.label(format!("{:?}", app_state.home.balance));

    ui.heading("Transaction History");
    app_state.home.transactions.iter().for_each(|t| {
        ui.label(format!("{:?}", t));
    });
}
