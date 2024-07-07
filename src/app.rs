use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};

use bdk_wallet::Wallet;
use sidepanel::sidepanel;

use crate::wallet::WalletBackground;
use crate::{bdk_utils, messages};

const DEFAULT_WORDS: &str =
    "rigid electric alert high ethics mystery pear reform alley height repeat manual";

pub struct WalletApp {
    pub page: Page,
    pub debug: String,
    pub wallet_info: WalletInfo,
    pub counter: i32,
    pub settings: Settings,
    pub wallet: Arc<Mutex<Wallet>>,
    pub wallet_req: Sender<messages::WalletRequest>,
    pub wallet_updates: Receiver<messages::WalletResponse>,
}

#[derive(Debug)]
pub struct WalletInfo {
    pub wallet_words: String,
    pub name: String,
}

impl WalletInfo {
    fn from_wallet() -> Self {
        Self {
            wallet_words: DEFAULT_WORDS.into(),
            name: "test".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub electrum_url: String,
    pub wallet_db: String,
}

impl Settings {
    fn new() -> Self {
        Self {
            electrum_url: "".into(),
            wallet_db: "".into(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Page {
    Home,
    Transaction,
    Settings,
}

mod home;
mod settings;
mod sidepanel;
mod transactions;

impl WalletApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let req: (
            Sender<messages::WalletRequest>,
            Receiver<messages::WalletRequest>,
        ) = mpsc::channel();
        let resp: (
            Sender<messages::WalletResponse>,
            Receiver<messages::WalletResponse>,
        ) = mpsc::channel();
        std::thread::spawn(move || {
            let recv = req.1;
            let send = resp.0;
            let bg = WalletBackground::new(recv, send);
            bg.monitor_wallet();
        });

        let wallet = Arc::new(Mutex::new(bdk_utils::create_new()));

        WalletApp {
            page: Page::Home,
            debug: "".into(),
            counter: 0,
            wallet,
            wallet_info: WalletInfo::from_wallet(),
            settings: Settings::new(),
            wallet_req: req.0,
            wallet_updates: resp.1,
        }
    }
}

impl eframe::App for WalletApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // eframe::set_value(storage, eframe::APP_KEY, self);
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
