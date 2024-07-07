use std::{
    sync::mpsc::{Receiver, Sender},
    thread,
    time::Duration,
};

use crate::messages::{self, WalletRequest, WalletResponse};

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
}

pub fn monitor_wallet(wallet: WalletBackground) {
    loop {
        thread::sleep(Duration::from_millis(500));
        let req = wallet.wallet_req.try_recv();
        if let Ok(req) = req {
            match req {
                WalletRequest::Echo(e) => wallet
                    .wallet_updates
                    .send(WalletResponse::Echo(e))
                    .expect("Main thread stopped"),
            };
        };
    }
}
