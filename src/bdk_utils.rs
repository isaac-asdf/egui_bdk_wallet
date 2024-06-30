use bdk::bitcoin::Network;
use bdk::database::MemoryDatabase;
use bdk::keys::{
    bip39::{Language, Mnemonic, WordCount},
    DerivableKey, ExtendedKey, GeneratableKey, GeneratedKey,
};
use bdk::template::Bip84;
use bdk::{miniscript, KeychainKind, Wallet};

pub fn create_new() -> Wallet<MemoryDatabase> {
    // Generate fresh mnemonic
    let mnemonic: GeneratedKey<_, miniscript::Segwitv0> =
        Mnemonic::generate((WordCount::Words12, Language::English)).unwrap();
    // Convert mnemonic to string
    let mnemonic_words = mnemonic.to_string();
    // Parse a mnemonic
    let mnemonic = Mnemonic::parse(&mnemonic_words).unwrap();
    // Generate the extended key
    let xkey: ExtendedKey = mnemonic.into_extended_key().unwrap();
    finalize_wallet(xkey)
}

pub fn from_words(words: Mnemonic) -> Wallet<MemoryDatabase> {
    // Generate the extended key
    let xkey: ExtendedKey = words.into_extended_key().unwrap();
    finalize_wallet(xkey)
}

fn finalize_wallet(xkey: ExtendedKey) -> Wallet<MemoryDatabase> {
    let network = Network::Testnet; // Or this can be Network::Bitcoin, Network::Signet or Network::Regtest
                                    // Create a BDK wallet structure using BIP 84 descriptor ("m/84h/1h/0h/0" and "m/84h/1h/0h/1")
                                    // Get xprv from the extended key
    let xprv = xkey.into_xprv(network).unwrap();
    let wallet = Wallet::new(
        Bip84(xprv, KeychainKind::External),
        Some(Bip84(xprv, KeychainKind::Internal)),
        network,
        MemoryDatabase::default(),
    )
    .unwrap();
    return wallet;
}
