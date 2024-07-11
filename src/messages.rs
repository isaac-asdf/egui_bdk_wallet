use bdk_wallet::{wallet::Balance, Wallet};

use crate::app::Settings;

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

pub enum WalletRequest {
    Debug(String),
    Sync,
    CreateNew(Wallet),
    AppConfig(AppConfig),
}

// pub struct WalletResponse {
//     pub status: i32,
// }
pub enum WalletResponse {
    Debug(String),
    Sync(Balance),
}
