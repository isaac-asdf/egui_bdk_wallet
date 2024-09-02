use bdk_sqlite::rusqlite::Connection;
use bdk_wallet::{
    bitcoin::{Address, Psbt},
    AddressInfo, Balance, LocalOutput, PersistedWallet,
};

use crate::app::{send::SendState, settings::Settings};

pub struct CreatedWallet {
    pub wallet: PersistedWallet<Connection>,
    pub name: String,
}

pub struct TxParts {
    pub sats_amount: u64,
    pub addr: Address,
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
            addr: value.pay_to_addr.unwrap(),
            utxos,
        }
    }
}

pub enum WalletRequest {
    Debug(String),
    Sync,
    AppConfig(Settings),
    CreateTransaction(TxParts),
    SendTransaction(Psbt),
    MarkUsed(AddressInfo),
    Close,
}

pub enum WalletResponse {
    WalletReady,
    Debug(String),
    Sync(Balance),
    RecvAddresses(Vec<AddressInfo>),
    UtxoList(Vec<LocalOutput>),
    NewPsbt(Psbt),
}
