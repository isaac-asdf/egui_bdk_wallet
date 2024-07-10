// pub struct WalletRequest {
//     pub request: i32,
// }

use bdk_wallet::wallet::Balance;

pub enum WalletRequest {
    Debug(String),
    Sync(String),
}

// pub struct WalletResponse {
//     pub status: i32,
// }
pub enum WalletResponse {
    Debug(String),
    Sync(Balance),
}
