use std::collections::HashSet;
use std::io::Write;

use crate::{bdk_utils, WalletApp};

use bdk_electrum::electrum_client;
use bdk_electrum::BdkElectrumClient;
use bdk_wallet::{keys::bip39::Mnemonic, KeychainKind, Wallet};

const STOP_GAP: usize = 50;
const BATCH_SIZE: usize = 5;

pub fn home(app_state: &mut WalletApp, ui: &mut egui::Ui) {
    ui.heading("Home");

    if ui.button("Create Wallet from Words").clicked() {
        // Parse a mnemonic
        let words = Mnemonic::parse(&app_state.wallet_words);
        if let Ok(words) = words {
            app_state.wallet = bdk_utils::from_words(words);
        } else {
            app_state.wallet_words += " word parse failed";
        }
    }

    if ui.button("Sync").clicked() {
        let db_path = std::env::temp_dir().join("bdk-electrum-example");
        let mut db = bdk_file_store::Store::<bdk_wallet::wallet::ChangeSet>::open_or_create_new(
            b"magic_bytes",
            db_path,
        )
        .unwrap();

        let client = BdkElectrumClient::new(
            electrum_client::Client::new("ssl://electrum.blockstream.info:60002").unwrap(),
        );

        // Populate the electrum client's transaction cache so it doesn't redownload transaction we
        // already have.
        client.populate_tx_cache(&app_state.wallet);

        let request = app_state
            .wallet
            .start_full_scan()
            .inspect_spks_for_all_keychains({
                let mut once = HashSet::<KeychainKind>::new();
                move |k, spk_i, _| {
                    if once.insert(k) {
                        print!("\nScanning keychain [{:?}]", k)
                    } else {
                        print!(" {:<3}", spk_i)
                    }
                }
            })
            .inspect_spks_for_all_keychains(|_, _, _| {
                std::io::stdout().flush().expect("must flush")
            });
        let mut update = client
            .full_scan(request, STOP_GAP, BATCH_SIZE, false)
            .unwrap()
            .with_confirmation_time_height_anchor(&client)
            .unwrap();

        let now = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs();
        let _ = update.graph_update.update_last_seen_unconfirmed(now);

        app_state.wallet.apply_update(update).unwrap();
        if let Some(changeset) = app_state.wallet.take_staged() {
            db.append_changeset(&changeset).unwrap();
        }

        let balance = app_state.wallet.balance();
        app_state.debug = format!("Wallet balance after syncing: {} sats", balance.total());
    }

    if app_state.debug.len() > 0 {
        ui.label("Debug");
        ui.text_edit_multiline(&mut app_state.debug);
    }

    ui.label("Words");
    ui.text_edit_multiline(&mut app_state.wallet_words);
    ui.label("Wallet Info");
    ui.label(format!(
        "{:#?}",
        app_state
            .wallet
            .get_descriptor_for_keychain(KeychainKind::External)
            .to_string()
    ));
}
