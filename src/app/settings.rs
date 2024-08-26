use serde::{Deserialize, Serialize};

use crate::WalletApp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub electrum_url: String,
    pub wallet_db: String,
}

const FOLDER: &str = "bdkw";
const SETTINGS: &str = "settings.json";

impl Settings {
    pub fn new() -> Self {
        let mut dir = dirs::config_dir().expect("Unable to find config dir");
        dir.push(FOLDER);
        if !dir.exists() {
            std::fs::create_dir(dir.clone()).expect("unable to create config directory");
        }
        dir.push(SETTINGS);

        if !dir.exists() {
            dir.push("wallets");
            Self {
                electrum_url: "ssl://electrum.blockstream.info:60002".into(),
                wallet_db: dir.to_str().unwrap().to_string(),
            }
        } else {
            let str = std::fs::read_to_string(dir).expect("already checked if exists");
            serde_json::from_str(&str).expect("invalid settings file detected")
        }
    }

    pub fn save(&self) {
        let mut dir = dirs::config_dir().expect("Unable to find config dir");
        dir.push(FOLDER);
        dir.push(SETTINGS);
        let file = std::fs::File::create(dir).unwrap();
        serde_json::to_writer_pretty(file, &self).unwrap();
    }
}

pub fn page(app_state: &mut WalletApp, ui: &mut egui::Ui) {
    ui.heading("Settings");

    ui.horizontal(|ui| {
        ui.label("Electrum URL: ");
        ui.text_edit_singleline(&mut app_state.settings.electrum_url);
    });

    ui.horizontal(|ui| {
        ui.label("DB URL: ");
        ui.text_edit_singleline(&mut app_state.settings.wallet_db);
        if ui.button("Save wallet").clicked() {
            app_state.settings.save();
            app_state
                .wallet_req
                .send(crate::messages::WalletRequest::AppConfig(
                    app_state.settings.clone().into(),
                ))
                .unwrap();
        }
    });
}
