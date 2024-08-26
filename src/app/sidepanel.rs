use crate::WalletApp;

use super::Page;

pub fn sidepanel(app_state: &mut WalletApp, ui: &mut egui::Ui) {
    if ui.button("Home").clicked() {
        app_state.page = Page::Home;
    }

    if ui.button("Send").clicked() {
        app_state.page = Page::Send;
    }

    if ui.button("Receive").clicked() {
        app_state.page = Page::Receive;
    }

    if ui.button("Transactions").clicked() {
        app_state.page = Page::Transactions;
    }
}
