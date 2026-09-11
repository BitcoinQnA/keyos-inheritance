use inheritance_core::{parse_wallet, Plan};
use miniscript::{
    bitcoin::{
        bip32::{Xpriv, Xpub},
        secp256k1::Secp256k1,
        Network,
    },
    descriptor::DescriptorPublicKey,
    Descriptor,
};
use std::str::FromStr;

pub fn keys(network: Network) -> Vec<String> {
    let secp = Secp256k1::new();
    (1..=3)
        .map(|i| {
            let root = Xpriv::new_master(network, &[i; 32]).unwrap();
            let xpub = Xpub::from_priv(&secp, &root);
            format!("[{}]{}", xpub.fingerprint(), xpub)
        })
        .collect()
}
pub fn checksum(raw: &str) -> String {
    Descriptor::<DescriptorPublicKey>::from_str(raw)
        .unwrap()
        .to_string()
}
pub fn descriptor() -> String {
    let keys = keys(Network::Testnet);
    checksum(&format!(
        "wsh(sortedmulti(2,{}/<0;1>/*,{}/<0;1>/*,{}/<0;1>/*))",
        keys[0], keys[1], keys[2]
    ))
}
pub fn plan() -> Plan {
    let mut p = Plan::new(&parse_wallet(&descriptor()).unwrap());
    p.wallet_name = "Family Vault (demo)".into();
    p.heir = "Alex Rivera".into();
    p.contact = "Sam — sam@example.com".into();
    p.message = "We practised this together. Take your time and ask Sam for help.".into();
    p.key_guidance = "Demo only:\n• Signer 1: home safe.\n• Signer 2: separate backup held by Sam.\n• Signer 3: Alex's device.\n\nMatch their fingerprints.".into();
    p.access_guidance = "A copy of this kit is with Sam. Arrange access to two signers using our separately stored instructions. No passwords are included here.".into();
    p
}
