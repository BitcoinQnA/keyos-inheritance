use inheritance_core::{export_kit, import_kit, parse_wallet, Plan};
use miniscript::{descriptor::DescriptorPublicKey, Descriptor};
use std::str::FromStr;

fn main() {
    let mut wallets = Vec::new();
    for path in std::env::args().skip(1) {
        let input = std::fs::read_to_string(&path).expect("Read export");
        match parse_wallet(&input) {
            Ok(wallet) => {
                println!(
                    "{path}: PASS\n{}\n{}",
                    wallet.signing_rules(),
                    wallet.network
                );
                let mut plan = Plan::new(&wallet);
                plan.upgrade().expect("Upgrade plan");
                assert_eq!(parse_wallet(&plan.bsms().unwrap()).unwrap(), wallet);
                let restored = import_kit(&export_kit(&plan).unwrap()).unwrap();
                assert_eq!(restored.validate().unwrap(), wallet);
                let original = if input.starts_with("BSMS") {
                    input.lines().nth(1).unwrap()
                } else {
                    input.trim()
                };
                let original = Descriptor::<DescriptorPublicKey>::from_str(original)
                    .unwrap()
                    .into_single_descriptors()
                    .unwrap();
                let recovered = Descriptor::<DescriptorPublicKey>::from_str(&restored.descriptor)
                    .unwrap()
                    .into_single_descriptors()
                    .unwrap();
                for branch in 0..2 {
                    for index in 0..100 {
                        assert_eq!(
                            original[branch]
                                .at_derivation_index(index)
                                .unwrap()
                                .script_pubkey(),
                            recovered[branch]
                                .at_derivation_index(index)
                                .unwrap()
                                .script_pubkey()
                        );
                    }
                }
                println!("100 receive and 100 change script comparisons: PASS");
                println!("BSMS and recovery-kit round trips: PASS");
                wallets.push(wallet);
            }
            Err(error) => panic!("{path}: REJECTED: {error}"),
        }
    }
    if wallets.len() == 2 {
        println!(
            "Both exports describe identical normalized wallets: {}",
            wallets[0] == wallets[1]
        );
    }
}
