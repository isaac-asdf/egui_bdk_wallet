use bdk_wallet::{bitcoin::Transaction, AddressInfo, Balance, LocalOutput, PersistedWallet};

use crate::app::{SendState, Settings};

pub struct AppConfig {
    pub wallets_loc: String,
    pub electrum_url: String,
}

impl From<Settings> for AppConfig {
    fn from(value: Settings) -> Self {
        Self {
            wallets_loc: value.wallet_db,
            electrum_url: value.electrum_url,
        }
    }
}

pub struct TxParts {
    pub sats_amount: u64,
    pub addr: String,
    pub utxos: Option<Vec<LocalOutput>>,
}

impl From<SendState> for TxParts {
    fn from(value: SendState) -> Self {
        let utxos = if value.selected_utxos.len() > 0 {
            Some(value.selected_utxos)
        } else {
            None
        };
        TxParts {
            sats_amount: value.sats_amount,
            addr: value.pay_to_addr,
            utxos,
        }
    }
}

pub enum WalletRequest {
    Debug(String),
    Sync,
    CreateNew(PersistedWallet),
    AppConfig(AppConfig),
    CreateTransaction(TxParts),
    SendTransaction(Transaction),
}

// pub struct WalletResponse {
//     pub status: i32,
// }
pub enum WalletResponse {
    Debug(String),
    Sync(Balance),
    RecvAddresses(Vec<AddressInfo>),
    UtxoList(Vec<LocalOutput>),
}
