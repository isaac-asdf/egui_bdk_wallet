use bdk_wallet::KeychainKind;

use super::WalletBackground;

pub fn get_unused_addrs(wallet: &mut WalletBackground) -> Vec<bdk_wallet::AddressInfo> {
    let mut addrs: Vec<bdk_wallet::AddressInfo> = Vec::new();
    let _ = wallet
        .wallet
        .list_unused_addresses(KeychainKind::External)
        .take_while(|next| {
            if addrs.len() > 10 {
                false
            } else {
                addrs.push(
                    wallet
                        .wallet
                        .peek_address(KeychainKind::External, next.index),
                );
                true
            }
        })
        .collect::<Vec<_>>();

    while addrs.len() <= 10 {
        addrs.push(wallet.wallet.reveal_next_address(KeychainKind::External));
    }
    wallet.persist();
    addrs
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     #[test]
//     fn get_addrs() {
//         todo!()
//     }
// }
