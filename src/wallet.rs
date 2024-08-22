use std::{
    str::FromStr,
    sync::mpsc::{Receiver, Sender},
    thread,
    time::Duration,
};

use bdk_wallet::{
    bitcoin::{Amount, FeeRate, ScriptBuf, Transaction, WPubkeyHash},
    Balance, PersistedWallet,
};

use crate::{
    bdk_utils,
    messages::{self, AppConfig, CreatedWallet, TxParts, WalletRequest, WalletResponse},
};

pub struct WalletBackground {
    wallet: Option<WalletInfo>,
    pub wallet_req: Receiver<messages::WalletRequest>,
    pub wallet_updates: Sender<messages::WalletResponse>,
}

struct WalletInfo {
    wallet: PersistedWallet,
    wallet_db: String,
}

impl WalletBackground {
    pub fn new(
        req: Receiver<messages::WalletRequest>,
        resp: Sender<messages::WalletResponse>,
    ) -> Self {
        WalletBackground {
            wallet: None,
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
                    WalletRequest::Sync => self.handle_sync(),
                    WalletRequest::CreateNew(w) => self.handle_new_wallet(w),
                    WalletRequest::AppConfig(c) => self.handle_config(c),
                    WalletRequest::SendTransaction(tx) => self.send_tx(tx),
                    WalletRequest::CreateTransaction(tx) => self.create_tx(tx),
                };
            };
        }
    }

    fn send_tx(&mut self, _tx: Transaction) {
        //
    }
    fn create_tx(&mut self, tx: TxParts) {
        self.wallet_updates
            .send(WalletResponse::Debug("Starting tx creation".into()))
            .unwrap();

        let wpkh: WPubkeyHash = WPubkeyHash::from_str(&tx.addr).unwrap();
        let script_pubkey = ScriptBuf::new_p2wpkh(&wpkh);
        let _tx = self
            .wallet
            .as_mut()
            .unwrap()
            .wallet
            .build_tx()
            .fee_rate(FeeRate::from_sat_per_vb(5_u64).unwrap())
            .add_recipient(script_pubkey, Amount::from_sat(tx.sats_amount));
    }

    fn handle_config(&mut self, c: AppConfig) {
        // self.wallet_db = c.wallets_loc;
        // self.electrum_url = c.electrum_url;
    }

    fn handle_new_wallet(&mut self, w: CreatedWallet) {
        self.wallet = Some(WalletInfo {
            wallet: w.wallet,
            wallet_db: w.name,
        });
        self.wallet_updates
            .send(WalletResponse::WalletReady)
            .unwrap();
    }

    fn handle_sync(&mut self) {
        // log starting
        self.wallet_updates
            .send(WalletResponse::Debug("Starting sync".into()))
            .unwrap();

        let refw = self.wallet.as_mut().unwrap();
        println!("got wallet");

        // request new state
        let cps: Vec<_> = refw.wallet.checkpoints().collect();
        let cp = refw.wallet.latest_checkpoint();
        let bal: Balance = if cps.len() > 1 {
            // short synce
            bdk_utils::cp_sync(cp, &refw.wallet_db, &mut refw.wallet)
        } else {
            // full synce
            bdk_utils::full_scan(&refw.wallet_db, &mut refw.wallet)
        };

        // send balance to UI thread
        self.wallet_updates
            .send(WalletResponse::Sync(bal))
            .expect("main thread died");

        // log complete
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
