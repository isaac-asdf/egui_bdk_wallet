// pub struct WalletRequest {
//     pub request: i32,
// }
pub enum WalletRequest {
    Echo(i32),
}

// pub struct WalletResponse {
//     pub status: i32,
// }
pub enum WalletResponse {
    Echo(i32),
}
