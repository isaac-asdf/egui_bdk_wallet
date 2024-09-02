use std::{thread, time::Duration};

use bdk_sqlite::rusqlite::Connection;
use flume::{Receiver, Sender};

use bdk_wallet::{
    bitcoin::{Amount, FeeRate, Psbt},
    AddressInfo, Balance, LocalOutput, PersistedWallet, SignOptions,
};

use crate::{
    app::settings::Settings,
    bdk_utils,
    messages::{self, TxParts, WalletRequest, WalletResponse},
};

mod receive;

pub struct WalletBackground {
    wallet: PersistedWallet<Connection>,
    name: String,
    wallet_req: Receiver<messages::WalletRequest>,
    wallet_updates: Sender<messages::WalletResponse>,
    electrum_url: String,
    db: String,
}

impl WalletBackground {
    pub fn new(
        wallet: PersistedWallet<Connection>,
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

    fn persist(&mut self) {
        bdk_utils::persist(&self.db, &self.name, &mut self.wallet);
    }

    fn mark_used(&mut self, addr: AddressInfo) {
        self.wallet
            .mark_used(bdk_wallet::KeychainKind::External, addr.index);
        self.persist();
    }

    fn get_utxos(&self) -> Vec<LocalOutput> {
        self.wallet.list_unspent().collect()
    }

    pub fn monitor_wallet(&mut self) {
        self.get_balance();
        let addr = receive::get_unused_addrs(self);
        self.wallet_updates
            .send(WalletResponse::RecvAddresses(addr))
            .unwrap();
        self.wallet_updates
            .send(WalletResponse::UtxoList(self.get_utxos()))
            .unwrap();

        // tell ui to go to loaded wallet display
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
                    WalletRequest::MarkUsed(addr) => self.mark_used(addr),
                    WalletRequest::Close => break,
                };
            };
        }
        println!("Closing wallet");
    }

    fn send_tx(&mut self, mut psbt: Psbt) {
        // fn send_tx(&mut self, tx: Transaction) {
        let sigops = SignOptions::default();
        let msg = if let Err(e) = self.wallet.finalize_psbt(&mut psbt, sigops) {
            e.to_string()
        } else {
            let tx = psbt.extract_tx().unwrap();
            let res = bdk_utils::broadcast_tx(&tx);
            match res {
                Ok(txid) => format!("txid: {txid:?}"),
                Err(e) => e,
            }
        };
        self.wallet_updates
            .send(WalletResponse::Debug(msg))
            .unwrap();
    }
    fn create_tx(&mut self, tx: TxParts) {
        self.wallet_updates
            .send(WalletResponse::Debug("Starting tx creation".into()))
            .unwrap();

        // let wpkh: WPubkeyHash = WPubkeyHash::from_str(&tx.addr).unwrap();
        // let script_pubkey = ScriptBuf::new_p2wpkh(&wpkh);
        let mut builder = self.wallet.build_tx();
        builder
            .fee_rate(FeeRate::from_sat_per_vb(5_u64).unwrap())
            .add_recipient(tx.addr.script_pubkey(), Amount::from_sat(tx.sats_amount));

        if let Some(selected) = tx.utxos {
            selected.iter().for_each(|utxo| {
                builder.add_utxo(utxo.outpoint).unwrap();
            })
        }

        let built = builder.finish();

        // self.persist();

        if let Ok(res) = built {
            self.wallet_updates
                .send(WalletResponse::NewPsbt(res))
                .unwrap();
            self.wallet_updates
                .send(WalletResponse::Debug("TX Created".into()))
                .unwrap();
        } else {
            self.wallet_updates
                .send(WalletResponse::Debug("tx creation failed".into()))
                .unwrap();
        }
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
