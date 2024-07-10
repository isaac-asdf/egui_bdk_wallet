use std::{
    sync::mpsc::{Receiver, Sender},
    thread,
    time::Duration,
};

use bdk_wallet::Wallet;

use crate::{
    bdk_utils,
    messages::{self, WalletRequest, WalletResponse},
};

pub struct WalletBackground {
    wallet: Wallet,
    pub wallet_req: Receiver<messages::WalletRequest>,
    pub wallet_updates: Sender<messages::WalletResponse>,
}

impl WalletBackground {
    pub fn new(
        wallet: Wallet,
        req: Receiver<messages::WalletRequest>,
        resp: Sender<messages::WalletResponse>,
    ) -> Self {
        WalletBackground {
            wallet,
            wallet_req: req,
            wallet_updates: resp,
        }
    }

    pub fn monitor_wallet(&mut self) {
        loop {
            thread::sleep(Duration::from_millis(500));
            let req = self.wallet_req.try_recv();
            if let Ok(req) = req {
                match req {
                    WalletRequest::Debug(s) => self.handle_debug(s),
                    WalletRequest::Sync(s) => self.handle_sync(s),
                };
            };
        }
    }

    fn handle_sync(&mut self, db_path: String) {
        let bal = bdk_utils::sync_db(&db_path, &mut self.wallet);
        self.wallet_updates
            .send(WalletResponse::Sync(bal))
            .expect("main thread died");
    }

    fn handle_debug(&self, s: String) {
        self.wallet_updates
            .send(WalletResponse::Debug(s))
            .expect("Main thread stopped")
    }
}
