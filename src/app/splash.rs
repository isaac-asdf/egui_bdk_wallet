use bdk_wallet::bitcoin::ecdsa::SerializedSignature;

use crate::{messages::WalletRequest, WalletApp};

const NEW_NAME: &str = "New";
pub fn page(app_state: &mut WalletApp, ui: &mut egui::Ui) {
    ui.heading("Select a wallet to begin:");
    egui::ComboBox::from_label("Select wallet")
        .selected_text(format!("{}", app_state.splash.selected_wallet))
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut app_state.splash.selected_wallet,
                NEW_NAME.into(),
                NEW_NAME,
            );
            app_state.splash.wallets.iter().for_each(|w| {
                ui.selectable_value(&mut app_state.splash.selected_wallet, w.to_string(), w);
            })
        });

    if &app_state.splash.selected_wallet == NEW_NAME {
        ui.heading("New wallet options:");
        egui::ComboBox::from_label("Create Option")
            .selected_text(format!("{:?}", app_state.splash.new_option))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut app_state.splash.new_option,
                    super::NewWallet::Seed,
                    "Seed",
                );
                ui.selectable_value(
                    &mut app_state.splash.new_option,
                    super::NewWallet::Xpub,
                    "Xpub",
                );
                ui.selectable_value(
                    &mut app_state.splash.new_option,
                    super::NewWallet::Descriptor,
                    "Descriptor",
                );
            });

        match app_state.splash.new_option {
            super::NewWallet::Seed => seed_opt(app_state, ui),
            super::NewWallet::Xpub => xpub_opt(app_state, ui),
            super::NewWallet::Descriptor => descriptor_opt(app_state, ui),
        }
    }
}

fn seed_opt(app_state: &mut WalletApp, ui: &mut egui::Ui) {
    ui.heading("Enter seed below:");
}

fn xpub_opt(app_state: &mut WalletApp, ui: &mut egui::Ui) {
    ui.heading("Enter xpub below:");
}

fn descriptor_opt(app_state: &mut WalletApp, ui: &mut egui::Ui) {
    ui.heading("Enter descriptor below:");
}
