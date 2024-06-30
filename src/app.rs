use bdk_wallet::Wallet;
use sidepanel::sidepanel;

use crate::bdk_utils;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct WalletApp {
    /// Currently viewed page
    #[serde(skip)] // This how you opt-out of serialization of a field
    page: Page,
    #[serde(skip)]
    wallet: Wallet,
    wallet_words: String,
    electrum_url: String,
    db_url: String,
    debug: String,
}

// #[derive(Debug, Copy, Clone, serde::Deserialize, serde::Serialize)]
// struct Wallet {
//     xpub:
// }

#[derive(Debug, Copy, Clone, serde::Deserialize, serde::Serialize)]
enum Page {
    Home,
    Transaction,
    Settings,
}

mod home;
mod settings;
mod sidepanel;
mod transactions;

impl Default for WalletApp {
    fn default() -> Self {
        Self {
            page: Page::Home,
            wallet: bdk_utils::create_new(),
            electrum_url: "".into(),
            db_url: "".into(),
            wallet_words:
                "rigid electric alert high ethics mystery pear reform alley height repeat manual"
                    .into(),
            debug: "".into(),
        }
    }
}

impl WalletApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for WalletApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::SidePanel::left("side").show(ctx, |ui| sidepanel(self, ui));

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.page {
                Page::Home => home::home(self, ui),
                Page::Transaction => transactions::transaction(self, ui),
                Page::Settings => settings::settings(self, ui),
            };

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}
