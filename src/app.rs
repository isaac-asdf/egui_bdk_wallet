use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use bdk_wallet::bitcoin::Transaction;
use bdk_wallet::wallet::Balance;
use sidepanel::sidepanel;

use crate::wallet::WalletBackground;
use crate::{bdk_utils, messages};

const DEFAULT_WORDS: &str =
    "rigid electric alert high ethics mystery pear reform alley height repeat manual";

mod home;
mod receive;
mod send;
mod settings;
mod sidepanel;
mod transactions;

pub struct WalletApp {
    /// Currently viewed page
    pub page: Page,
    /// for debug purposes
    pub debug: String,
    /// UI display for wallet info
    pub wallet_info: WalletInfo,
    /// State for Home page
    pub home: HomeState,
    /// State for Send page
    pub send: SendState,
    /// State for Receive page
    pub receive: ReceiveState,
    /// State data for settings page
    pub settings: Settings,
    /// Channel for requests to the wallet thread
    pub wallet_req: Sender<messages::WalletRequest>,
    /// Channel for updates from the wallet thread
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
pub struct HomeState {
    pub balance: Option<Balance>,
    pub transactions: Vec<Transaction>,
}

impl HomeState {
    fn new() -> Self {
        HomeState {
            balance: None,
            transactions: Vec::new(),
        }
    }
}

pub struct SendState {
    pub pay_to_addr: String,
    pub label: String,
    pub sats_amount: u64,
    pub selected_utxos: Vec<i32>,
    pub fee_rate: f32,
    pub fees: u64,
}

impl SendState {
    pub fn new() -> Self {
        SendState {
            pay_to_addr: "".into(),
            label: "".into(),
            sats_amount: 0,
            selected_utxos: Vec::new(),
            fee_rate: 1.,
            fees: 0,
        }
    }
}

pub struct ReceiveState {
    pub pay_to_addr: String,
    pub label: String,
    pub derivation: String,
    pub next_addr: Vec<String>,
}

impl ReceiveState {
    pub fn new() -> Self {
        ReceiveState {
            pay_to_addr: "".into(),
            label: "".into(),
            derivation: "".into(),
            next_addr: Vec::new(),
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
    Send,
    Receive,
    Transactions,
    Settings,
}

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
            let mut bg = WalletBackground::new(bdk_utils::create_new(), recv, send);
            bg.monitor_wallet();
        });

        WalletApp {
            page: Page::Home,
            debug: "".into(),
            wallet_info: WalletInfo::from_wallet(),
            home: HomeState::new(),
            send: SendState::new(),
            receive: ReceiveState::new(),
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
        let update = self.wallet_updates.try_recv();
        if let Ok(update) = update {
            // update state
            match update {
                messages::WalletResponse::Debug(s) => self.debug = s,
                messages::WalletResponse::Sync(b) => self.home.balance = Some(b),
            }
        }

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
                Page::Home => home::page(self, ui),
                Page::Send => send::page(self, ui),
                Page::Receive => receive::page(self, ui),
                Page::Transactions => transactions::page(self, ui),
                Page::Settings => settings::page(self, ui),
            };

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}
