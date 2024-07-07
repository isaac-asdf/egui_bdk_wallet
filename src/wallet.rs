use std::{
    sync::mpsc::{Receiver, Sender},
    thread,
    time::Duration,
};

use crate::{
    bdk_utils,
    messages::{self, Sync, WalletRequest, WalletResponse},
};

pub struct WalletBackground {
    pub wallet_req: Receiver<messages::WalletRequest>,
    pub wallet_updates: Sender<messages::WalletResponse>,
}

impl WalletBackground {
    pub fn new(
        req: Receiver<messages::WalletRequest>,
        resp: Sender<messages::WalletResponse>,
    ) -> Self {
        WalletBackground {
            wallet_req: req,
            wallet_updates: resp,
        }
    }

    pub fn monitor_wallet(&self) {
        loop {
            thread::sleep(Duration::from_millis(500));
            let req = self.wallet_req.try_recv();
            if let Ok(req) = req {
                match req {
                    WalletRequest::Echo(e) => self.handle_echo(e),
                    WalletRequest::Sync(s) => self.handle_sync(s),
                };
            };
        }
    }

    fn handle_sync(&self, s: crate::messages::Sync) {
        //
        let bal = bdk_utils::sync_db(&s.db_path, &mut s.wallet.lock().unwrap());
        self.wallet_updates
            .send(WalletResponse::Sync(bal))
            .expect("main thread died");
    }

    fn handle_echo(&self, e: i32) {
        self.wallet_updates
            .send(WalletResponse::Echo(e))
            .expect("Main thread stopped")
    }
}
