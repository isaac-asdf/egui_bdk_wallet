use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use sidepanel::sidepanel;

use crate::messages;
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
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let settings = settings::Settings::new();
        let cl = settings.clone();
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
            let mut bg = WalletBackground::new(cl.into(), recv, send);
            bg.monitor_wallet();
        });

        WalletApp {
            page: Page::SplashScreen,
            debug: Vec::new(),
            wallet_info: WalletInfo::from_wallet(),
            splash: splash::SplashState::new(),
            home: home::HomeState::new(),
            send: send::SendState::new(),
            receive: receive::ReceiveState::new(),
            settings,
            wallet_req: req.0,
            wallet_updates: resp.1,
        }
    }
}

impl eframe::App for WalletApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
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

        if self.page != Page::SplashScreen {
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
