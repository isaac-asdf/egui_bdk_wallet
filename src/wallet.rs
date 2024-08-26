use std::{str::FromStr, thread, time::Duration};

use flume::{Receiver, Sender};

use bdk_wallet::{
    bitcoin::{Amount, FeeRate, ScriptBuf, Transaction, WPubkeyHash},
    Balance, PersistedWallet,
};

use crate::{
    app::settings::Settings,
    bdk_utils,
    messages::{self, TxParts, WalletRequest, WalletResponse},
};

pub struct WalletBackground {
    wallet: PersistedWallet,
    name: String,
    wallet_req: Receiver<messages::WalletRequest>,
    wallet_updates: Sender<messages::WalletResponse>,
    electrum_url: String,
    db: String,
}

impl WalletBackground {
    pub fn new(
        wallet: PersistedWallet,
        name: String,
        req: Receiver<messages::WalletRequest>,
        resp: Sender<messages::WalletResponse>,
        settings: Settings,
    ) -> Self {
        WalletBackground {
            wallet,
            name,
            wallet_req: req,
            wallet_updates: resp,
            electrum_url: settings.electrum_url,
            db: settings.wallet_db,
        }
    }

    pub fn monitor_wallet(&mut self) {
        self.get_balance();
        self.get_unused_addrs();
        self.wallet_updates
            .send(messages::WalletResponse::WalletReady)
            .unwrap();
        loop {
            thread::sleep(Duration::from_millis(500));
            let req = self.wallet_req.try_recv();
            if let Ok(req) = req {
                match req {
                    WalletRequest::Debug(s) => self.handle_debug(s),
                    WalletRequest::Sync => self.handle_sync(),
                    WalletRequest::AppConfig(c) => self.handle_config(c),
                    WalletRequest::SendTransaction(tx) => self.send_tx(tx),
                    WalletRequest::CreateTransaction(tx) => self.create_tx(tx),
                    WalletRequest::Close => break,
                };
            };
        }
        println!("Closing wallet");
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
            .build_tx()
            .fee_rate(FeeRate::from_sat_per_vb(5_u64).unwrap())
            .add_recipient(script_pubkey, Amount::from_sat(tx.sats_amount));
    }

    fn handle_config(&mut self, c: Settings) {
        self.db = c.wallet_db;
        self.electrum_url = c.electrum_url;
    }

    fn get_balance(&self) {
        let balance = self.wallet.balance();
        self.wallet_updates
            .send(WalletResponse::Sync(balance))
            .unwrap();
    }

    fn get_unused_addrs(&mut self) {
        // let revealed = self
        //     .wallet
        //     .list_unused_addresses(bdk_wallet::KeychainKind::External)
        //     .collect();
        // let addrs = vec![wallet.reveal_next_address(bdk_wallet::KeychainKind::External)];
        // self.wallet_updates
        // .send(WalletResponse::RecvAddresses(revealed))
        // .unwrap();
    }

    fn handle_sync(&mut self) {
        // log starting
        self.wallet_updates
            .send(WalletResponse::Debug("Starting sync".into()))
            .unwrap();

        // request new state
        let cps: Vec<_> = self.wallet.checkpoints().collect();
        let bal: Balance = if cps.len() > 1 {
            // short synce
            bdk_utils::cp_sync(&self.db, &self.name, &mut self.wallet)
        } else {
            // full synce
            bdk_utils::full_scan(&self.db, &self.name, &mut self.wallet)
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
