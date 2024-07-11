use std::{
    sync::mpsc::{Receiver, Sender},
    thread,
    time::Duration,
};

use bdk_wallet::Wallet;

use crate::{
    bdk_utils,
    messages::{self, AppConfig, WalletRequest, WalletResponse},
};

pub struct WalletBackground {
    wallet: Wallet,
    wallet_db: String,
    electrum_url: String,
    pub wallet_req: Receiver<messages::WalletRequest>,
    pub wallet_updates: Sender<messages::WalletResponse>,
}

impl WalletBackground {
    pub fn new(
        config: AppConfig,
        req: Receiver<messages::WalletRequest>,
        resp: Sender<messages::WalletResponse>,
    ) -> Self {
        let wallet = bdk_utils::from_changeset(&config.wallets_loc);
        let wallet = match wallet {
            Ok(w) => w,
            Err(_) => bdk_utils::create_new(),
        };
        WalletBackground {
            wallet,
            wallet_db: config.wallets_loc,
            electrum_url: config.electrum_url,
            wallet_req: req,
            wallet_updates: resp,
        }
    }

    pub fn monitor_wallet(&mut self) {
        self.wallet_updates
            .send(WalletResponse::Sync(self.wallet.balance()))
            .expect("main stopped");
        loop {
            thread::sleep(Duration::from_millis(500));
            let req = self.wallet_req.try_recv();
            if let Ok(req) = req {
                match req {
                    WalletRequest::Debug(s) => self.handle_debug(s),
                    WalletRequest::Sync => self.handle_sync(),
                    WalletRequest::CreateNew(w) => self.handle_new_wallet(w),
                    WalletRequest::AppConfig(c) => self.handle_config(c),
                };
            };
        }
    }

    fn handle_config(&mut self, c: AppConfig) {
        self.wallet_db = c.wallets_loc;
        self.electrum_url = c.electrum_url;
    }

    fn handle_new_wallet(&mut self, w: Wallet) {
        self.wallet = w;
    }

    fn handle_sync(&mut self) {
        self.wallet_updates
            .send(WalletResponse::Debug("Starting sync".into()))
            .unwrap();
        let bal = bdk_utils::sync_db(&self.wallet_db, &mut self.wallet);
        self.wallet_updates
            .send(WalletResponse::Sync(bal))
            .expect("main thread died");
        self.wallet_updates
            .send(WalletResponse::Debug("Sync complete".into()))
            .unwrap();
    }

    fn handle_debug(&self, s: String) {
        self.wallet_updates
            .send(WalletResponse::Debug(s))
            .expect("Main thread stopped")
    }
}
