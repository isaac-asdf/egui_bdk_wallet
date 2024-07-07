// pub struct WalletRequest {
//     pub request: i32,
// }

use std::sync::{Arc, Mutex};

use bdk_wallet::{wallet::Balance, Wallet};

pub struct Sync {
    pub wallet: Arc<Mutex<Wallet>>,
    pub db_path: String,
}

pub enum WalletRequest {
    Echo(i32),
    Sync(Sync),
}

// pub struct WalletResponse {
//     pub status: i32,
// }
pub enum WalletResponse {
    Echo(i32),
    Sync(Balance),
}
