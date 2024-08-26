use flume::{Receiver, Sender};
use sidepanel::sidepanel;

use crate::messages::{self, CreatedWallet};
use crate::wallet::WalletBackground;

mod home;
mod receive;
pub mod send;
pub mod settings;
mod sidepanel;
mod splash;
mod transactions;

pub struct WalletApp {
    /// Currently viewed page
    pub page: Page,
    /// for debug purposes
    pub debug: Vec<String>,
    /// UI display for wallet info
    pub wallet_info: WalletInfo,
    /// State for Splash Screen
    pub splash: splash::SplashState,
    /// State for Home page
    pub home: home::HomeState,
    /// State for Send page
    pub send: send::SendState,
    /// State for Receive page
    pub receive: receive::ReceiveState,
    /// State data for settings page
    pub settings: settings::Settings,
    /// Channel for requests to the wallet thread
    pub wallet_req: Sender<messages::WalletRequest>,
    /// Channel for updates from the wallet thread
    pub wallet_updates: Receiver<messages::WalletResponse>,
    /// For cloning and sending to background worker thread
    for_bg_req: Receiver<messages::WalletRequest>,
    /// For cloning and sending to background worker thread
    for_bg_upd: Sender<messages::WalletResponse>,
}

#[derive(Debug)]
pub struct WalletInfo {
    pub name: String,
}

impl WalletInfo {
    fn from_wallet() -> Self {
        Self {
            name: "test".into(),
        }
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Page {
    SplashScreen,
    Home,
    Send,
    Receive,
    Transactions,
    Settings,
}

impl WalletApp {
    pub fn new_bg(&self, wallet: CreatedWallet) {
        let recv = self.for_bg_req.clone();
        let send = self.for_bg_upd.clone();
        let settings = self.settings.clone();
        std::thread::spawn(move || {
            let mut bg = WalletBackground::new(wallet.wallet, wallet.name, recv, send, settings);
            bg.monitor_wallet();
        });
    }

    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let req: (
            Sender<messages::WalletRequest>,
            Receiver<messages::WalletRequest>,
        ) = flume::unbounded();
        let resp: (
            Sender<messages::WalletResponse>,
            Receiver<messages::WalletResponse>,
        ) = flume::unbounded();
        let settings = settings::Settings::new();

        WalletApp {
            page: Page::SplashScreen,
            debug: Vec::new(),
            wallet_info: WalletInfo::from_wallet(),
            splash: splash::SplashState::new(&settings.wallet_db),
            home: home::HomeState::new(),
            send: send::SendState::new(),
            receive: receive::ReceiveState::new(),
            wallet_req: req.0,
            wallet_updates: resp.1,
            for_bg_req: req.1,
            for_bg_upd: resp.0,
            settings,
        }
    }
}

impl eframe::App for WalletApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // eframe::set_value(storage, eframe::APP_KEY, self);
        self.settings.save();
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // check for updates from background thread to update wallet state
        let update = self.wallet_updates.try_recv();
        if let Ok(update) = update {
            // update state
            match update {
                messages::WalletResponse::Debug(s) => {
                    self.debug.push(s);
                    if self.debug.len() > 5 {
                        self.debug.remove(0);
                    }
                }
                messages::WalletResponse::Sync(b) => self.home.balance = Some(b),
                messages::WalletResponse::UtxoList(utxos) => self.send.selected_utxos = utxos,
                messages::WalletResponse::RecvAddresses(addrs) => self.receive.next_addr = addrs,
                messages::WalletResponse::WalletReady => self.page = Page::Home,
            }
        }

        // Draw app

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("Change Wallet").clicked() {
                            let _ = self.wallet_req.send(messages::WalletRequest::Close);
                            self.page = Page::SplashScreen;
                            self.splash = splash::SplashState::new(&self.settings.wallet_db);
                        }
                        if ui.button("Settings").clicked() {
                            self.page = Page::Settings;
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        if self.page != Page::SplashScreen && self.page != Page::Settings {
            egui::SidePanel::left("side").show(ctx, |ui| sidepanel(self, ui));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.page {
                Page::SplashScreen => splash::page(self, ui),
                Page::Home => home::page(self, ui),
                Page::Send => send::page(self, ui),
                Page::Receive => receive::page(self, ui),
                Page::Transactions => transactions::page(self, ui),
                Page::Settings => settings::page(self, ui),
            };

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                self.debug.iter().for_each(|s| {
                    ui.label(s).highlight();
                });
                egui::warn_if_debug_build(ui);
            });
        });
    }
}
