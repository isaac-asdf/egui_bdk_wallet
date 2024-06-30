use crate::WalletApp;

use super::Page;

pub fn sidepanel(app_state: &mut WalletApp, ui: &mut egui::Ui) {
    if ui.button("Home").clicked() {
        app_state.page = Page::Home;
    }

    if ui.button("Transaction").clicked() {
        app_state.page = Page::Transaction;
    }

    if ui.button("Settings").clicked() {
        app_state.page = Page::Settings;
    }
}
