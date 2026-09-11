use inheritance_core::*;
use miniscript::{
    bitcoin::{
        bip32::{Xpriv, Xpub},
        secp256k1::Secp256k1,
        Network,
    },
    descriptor::checksum::desc_checksum,
};

fn descriptor(lock: &str) -> String {
    let keys: Vec<_> = (1..=6)
        .map(|i| {
            let private = Xpriv::new_master(Network::Testnet, &[i; 32]).unwrap();
            let public = Xpub::from_priv(&Secp256k1::new(), &private);
            format!("[{}]{public}/<0;1>/*", public.fingerprint())
        })
        .collect();
    signed(&format!(
        "wsh(or_d(multi(2,{}),and_v(v:multi(1,{}),{lock})))",
        keys[..3].join(","),
        keys[3..].join(",")
    ))
}
fn signed(body: &str) -> String {
    format!("{body}#{}", desc_checksum(body).unwrap())
}

#[test]
fn two_paths_and_time_are_not_flattened() {
    let w = parse_wallet(&descriptor("after(1790410829)")).unwrap();
    assert_eq!(w.threshold, 2);
    let p = w.inheritance.as_ref().unwrap();
    assert_eq!((p.threshold, p.normal_count, p.after), (1, 3, 1790410829));
    assert!(w.summary().contains("Inheritance: 1 of 3"));
    assert!(!w.summary().contains("2 of 6"));
    assert!(w.signing_rules().contains("median time"));
}
#[test]
fn height_is_not_a_date() {
    let w = parse_wallet(&descriptor("after(900000)")).unwrap();
    assert!(w.signing_rules().contains("block height 900000"));
}

#[test]
fn reordered_cards_cannot_mislabel_signing_groups() {
    let mut p = Plan::new(&parse_wallet(&descriptor("after(1790410829)")).unwrap());
    p.upgrade().unwrap();
    p.care.as_mut().unwrap().signers.swap(0, 3);
    assert!(p.validate().is_err());
}
#[test]
fn unsupported_relative_lock_fails_closed() {
    assert!(parse_wallet(&descriptor("older(144)")).is_err());
}
#[test]
fn bsms_no_restrictions_validates_address_and_checksum() {
    let d = descriptor("after(1790410829)");
    let w = parse_wallet(&d).unwrap();
    let b = format!("BSMS 1.0\n{d}\nNo path restrictions\n{}", w.first_address);
    assert_eq!(parse_wallet(&b).unwrap(), w);
    assert!(parse_wallet(&b.replace(&w.first_address, "tb1qwrong")).is_err());
    assert!(parse_wallet(&b.replace('#', "#bad")).is_err());
}
#[test]
fn template_without_restrictions_is_rejected() {
    let d = descriptor("after(1790410829)");
    let w = parse_wallet(&d).unwrap();
    let template = signed(&d.split('#').next().unwrap().replace("/<0;1>/*", "/**"));
    assert!(parse_wallet(&format!(
        "BSMS 1.0\n{template}\nNo path restrictions\n{}",
        w.first_address
    ))
    .is_err());
}
#[test]
fn each_path_needs_its_own_instructions() {
    let mut p = Plan::new(&parse_wallet(&descriptor("after(1790410829)")).unwrap());
    p.upgrade().unwrap();
    p.wallet_name = "Test".into();
    p.heir = "Heir".into();
    p.contact = "Contact".into();
    p.access_guidance = "Access".into();
    for s in &mut p.care.as_mut().unwrap().signers[..2] {
        s.location = "Location".into();
        s.access = "Access".into();
    }
    assert_eq!(p.completeness().len(), 1);
    assert!(p.completeness()[0].contains("Inheritance path"));
    p.care.as_mut().unwrap().signers[3].location = "Location".into();
    p.care.as_mut().unwrap().signers[3].access = "Access".into();
    assert!(p.completeness().is_empty());
    let restored = import_kit(&export_kit(&p).unwrap()).unwrap();
    assert_eq!(restored, p);
    assert!(p.guide().unwrap().contains("Inheritance: 1 of 3"));
    assert_eq!(
        parse_wallet(&p.bsms().unwrap()).unwrap(),
        p.validate().unwrap()
    );
}
