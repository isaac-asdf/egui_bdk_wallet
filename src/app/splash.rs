use bdk_wallet::bip39::Mnemonic;

use crate::bdk_utils;
use crate::messages::CreatedWallet;
use crate::{messages::WalletRequest, WalletApp};

#[derive(Debug)]
pub struct SplashState {
    selected_wallet: String,
    wallets: Vec<String>,
    new_name: String,
    new_1: String,
    new_2: String,
    new_option: NewWallet,
}

#[derive(PartialEq, Debug, Clone)]
pub enum NewWallet {
    Seed,
    Xpub,
    Descriptor,
}

impl SplashState {
    pub fn new() -> Self {
        SplashState {
            selected_wallet: String::new(),
            wallets: bdk_utils::list_wallets(),
            new_name: String::new(),
            new_1: String::new(),
            new_2: String::new(),
            new_option: NewWallet::Seed,
        }
    }
}

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
    if &app_state.splash.selected_wallet != "" && &app_state.splash.selected_wallet != NEW_NAME {
        if ui.button("Load current wallet").clicked() {
            // load wallet and send to backend on click
            let wallet = bdk_utils::from_changeset(&app_state.splash.selected_wallet);
            if let Ok(wallet) = wallet {
                let wallet = CreatedWallet {
                    wallet,
                    name: app_state.splash.new_name.clone(),
                };
                app_state
                    .wallet_req
                    .send(WalletRequest::CreateNew(wallet))
                    .unwrap();
            }
        }
    }

    if &app_state.splash.selected_wallet == NEW_NAME {
        ui.heading("New wallet options:");
        ui.horizontal(|ui| {
            ui.label("Enter name:");
            ui.text_edit_singleline(&mut app_state.splash.new_name);
        });
        egui::ComboBox::from_label("Create Option")
            .selected_text(format!("{:?}", app_state.splash.new_option))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut app_state.splash.new_option, NewWallet::Seed, "Seed");
                ui.selectable_value(&mut app_state.splash.new_option, NewWallet::Xpub, "Xpub");
                ui.selectable_value(
                    &mut app_state.splash.new_option,
                    NewWallet::Descriptor,
                    "Descriptor",
                );
            });

        match app_state.splash.new_option {
            NewWallet::Seed => seed_opt(app_state, ui),
            NewWallet::Xpub => xpub_opt(app_state, ui),
            NewWallet::Descriptor => descriptor_opt(app_state, ui),
        }
    }
}

fn seed_opt(app_state: &mut WalletApp, ui: &mut egui::Ui) {
    ui.heading("Enter seed below:");
    if ui.button("Give me a new seed please").clicked() {
        // fill in seed
        let new_seed = bdk_utils::new_seed();
        app_state.splash.new_1 = new_seed.to_string();
    }
    ui.text_edit_multiline(&mut app_state.splash.new_1);
    ui.heading("Confirm seed");
    ui.text_edit_multiline(&mut app_state.splash.new_2);
    if &app_state.splash.new_name != "" {
        if &app_state.splash.new_1 != "" && app_state.splash.new_1 == app_state.splash.new_2 {
            if ui.button("Proceed to load wallet").clicked() {
                //
                let mne = Mnemonic::parse(&app_state.splash.new_1);
                match mne {
                    Ok(seed) => finalize_wallet(app_state, seed),
                    Err(e) => {
                        app_state.splash.new_1 = "Invalid seed, try again...".into();
                        app_state.splash.new_2 = e.to_string();
                    }
                }
            }
        }
    }
}

fn finalize_wallet(state: &mut WalletApp, mne: Mnemonic) {
    let wallet = bdk_utils::from_words(&state.splash.new_name, mne);
    let wallet = CreatedWallet {
        wallet,
        name: state.splash.new_name.clone(),
    };
    state
        .wallet_req
        .send(WalletRequest::CreateNew(wallet))
        .unwrap();
}

fn xpub_opt(app_state: &mut WalletApp, ui: &mut egui::Ui) {
    ui.heading("Enter xpub below:");
    ui.text_edit_singleline(&mut app_state.splash.new_1);
    if &app_state.splash.new_1 != "" {
        if ui.button("Proceed to load wallet").clicked() {
            //
        }
    }
}

fn descriptor_opt(app_state: &mut WalletApp, ui: &mut egui::Ui) {
    ui.heading("Enter public descriptor below:");
    ui.text_edit_singleline(&mut app_state.splash.new_1);
    ui.heading("Enter change descriptor below:");
    ui.text_edit_singleline(&mut app_state.splash.new_2);
    if &app_state.splash.new_1 != "" && &app_state.splash.new_2 != "" {
        if ui.button("Proceed to load wallet").clicked() {
            //
        }
    }
}
