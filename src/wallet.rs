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
    messages::{self, AppConfig, TxParts, WalletRequest, WalletResponse},
};

pub struct WalletBackground {
    wallet: PersistedWallet,
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
            Err(_) => bdk_utils::create_new().0,
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

        self.wallet_updates
            .send(WalletResponse::RecvAddresses(vec![self
                .wallet
                .reveal_next_address(bdk_wallet::KeychainKind::External)]))
            .unwrap();

        let mut b_ons = true;
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

            if b_ons {
                b_ons = false;
                let utxos = self.wallet.list_unspent();
                let utxos = utxos.collect();
                let _ = self.wallet_updates.send(WalletResponse::UtxoList(utxos));
            }
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
            .build_tx()
            .fee_rate(FeeRate::from_sat_per_vb(5_u64).unwrap())
            .add_recipient(script_pubkey, Amount::from_sat(tx.sats_amount));
    }

    fn handle_config(&mut self, c: AppConfig) {
        self.wallet_db = c.wallets_loc;
        self.electrum_url = c.electrum_url;
    }

    fn handle_new_wallet(&mut self, w: PersistedWallet) {
        self.wallet = w;
    }

    fn handle_sync(&mut self) {
        // log starting
        self.wallet_updates
            .send(WalletResponse::Debug("Starting sync".into()))
            .unwrap();

        // request new state
        let cps: Vec<_> = self.wallet.checkpoints().collect();
        let cp = self.wallet.latest_checkpoint();
        let bal: Balance = if cps.len() > 1 {
            // short synce
            bdk_utils::cp_sync(cp, &self.wallet_db, &mut self.wallet)
        } else {
            // full synce
            bdk_utils::full_scan(&self.wallet_db, &mut self.wallet)
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
