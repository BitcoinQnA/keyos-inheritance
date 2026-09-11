mod support;
use inheritance_core::*;
use miniscript::bitcoin::Network;
use support::*;

#[test]
fn public_descriptor_reports_threshold_and_network() {
    let w = parse_wallet(&descriptor()).unwrap();
    assert_eq!(w.threshold, 2);
    assert_eq!(w.fingerprints.len(), 3);
    assert!(w.network.starts_with("Test"));
    assert!(w.first_address.starts_with("tb1q"));
}
#[test]
fn bsms_matches_first_address() {
    let w = parse_wallet(&descriptor()).unwrap();
    let input = format!("BSMS 1.0\n{}\n/0/*,/1/*\n{}", descriptor(), w.first_address);
    assert_eq!(parse_wallet(&input).unwrap(), w);
    assert!(parse_wallet(&input.replace(&w.first_address, "tb1qwrong")).is_err());
}
#[test]
fn bsms_accepts_crlf_and_trailing_newline() {
    let w = parse_wallet(&descriptor()).unwrap();
    assert!(parse_wallet(&format!(
        "BSMS 1.0\r\n{}\r\n/0/*,/1/*\r\n{}\r\n",
        descriptor(),
        w.first_address
    ))
    .is_ok());
}
#[test]
fn descriptor_requires_correct_checksum() {
    let d = descriptor();
    assert!(parse_wallet(d.split('#').next().unwrap()).is_err());
    assert!(parse_wallet(&format!("{}#aaaaaaaa", d.split('#').next().unwrap())).is_err());
}
#[test]
fn mismatched_bsms_version_and_paths_rejected() {
    let w = parse_wallet(&descriptor()).unwrap();
    for (version, paths) in [("2.0", "/0/*,/1/*"), ("1.0", "/1/*,/0/*")] {
        assert!(parse_wallet(&format!(
            "BSMS {version}\n{}\n{paths}\n{}",
            descriptor(),
            w.first_address
        ))
        .is_err());
    }
}
#[test]
fn private_keys_are_rejected() {
    let private =
        miniscript::bitcoin::bip32::Xpriv::new_master(Network::Testnet, &[9; 32]).unwrap();
    let public = keys(Network::Testnet);
    let raw = format!(
        "wsh(sortedmulti(2,[12345678]{private}/<0;1>/*,{}/<0;1>/*))",
        public[0]
    );
    let sum = miniscript::descriptor::checksum::desc_checksum(&raw).unwrap();
    assert!(parse_wallet(&format!("{raw}#{sum}")).is_err());
    assert!(parse_wallet("wsh(sortedmulti(2,xprv123/0/*,xprv456/0/*))#abcdefgh").is_err());
    assert!(parse_wallet("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about").is_err());
}
#[test]
fn malformed_input_smoke_fuzz_does_not_panic() {
    let mut rng = 7u64;
    for length in 0..1024 {
        let bytes: Vec<u8> = (0..length)
            .map(|_| {
                rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
                (rng >> 32) as u8
            })
            .collect();
        assert!(import_kit(&bytes).is_err());
        if let Ok(text) = std::str::from_utf8(&bytes) {
            assert!(parse_wallet(text).is_err());
        }
    }
}
#[test]
fn duplicate_signers_rejected() {
    let k = keys(Network::Testnet);
    let d = checksum(&format!(
        "wsh(sortedmulti(2,{}/<0;1>/*,{}/<0;1>/*))",
        k[0], k[0]
    ));
    assert!(parse_wallet(&d).is_err());
}
#[test]
fn mixed_networks_rejected() {
    let a = keys(Network::Bitcoin);
    let b = keys(Network::Testnet);
    let d = checksum(&format!(
        "wsh(sortedmulti(2,{}/<0;1>/*,{}/<0;1>/*))",
        a[0], b[1]
    ));
    assert!(parse_wallet(&d).unwrap_err().contains("mixes"));
}
#[test]
fn mainnet_supported() {
    let k = keys(Network::Bitcoin);
    let d = checksum(&format!(
        "wsh(sortedmulti(2,{}/<0;1>/*,{}/<0;1>/*,{}/<0;1>/*))",
        k[0], k[1], k[2]
    ));
    assert_eq!(parse_wallet(&d).unwrap().network, "Bitcoin mainnet");
}
#[test]
fn receive_only_rejected_to_preserve_change() {
    let d = descriptor();
    let receive = checksum(&d.split('#').next().unwrap().replace("/<0;1>/*", "/0/*"));
    assert!(parse_wallet(&receive).unwrap_err().contains("change"));
}
#[test]
fn bsms_template_expands_both_branches() {
    let w = parse_wallet(&descriptor()).unwrap();
    let template = w
        .descriptor
        .split('#')
        .next()
        .unwrap()
        .replace("/<0;1>/*", "/**");
    let input = format!("BSMS 1.0\n{template}\n/0/*,/1/*\n{}", w.first_address);
    assert_eq!(parse_wallet(&input).unwrap(), w);
    let sum = miniscript::descriptor::checksum::desc_checksum(&template).unwrap();
    let signed = input.replace(&template, &format!("{template}#{sum}"));
    assert_eq!(parse_wallet(&signed).unwrap(), w);
    assert!(parse_wallet(&signed.replace('#', "#wrong")).is_err());
    assert!(parse_wallet(&template).is_err());
}
#[test]
fn exported_bsms_preserves_complete_wallet() {
    let p = plan();
    assert_eq!(
        parse_wallet(&p.bsms().unwrap()).unwrap(),
        p.validate().unwrap()
    );
}
#[test]
fn bip129_public_reference_vector() {
    let input = "BSMS 1.0\nwsh(sortedmulti(2,[1cf0bf7e/48'/0'/0'/2']xpub6FL8FhxNNUVnG64YurPd16AfGyvFLhh7S2uSsDqR3Qfcm6o9jtcMYwh6DvmcBF9qozxNQmTCVvWtxLpKTnhVLN3Pgnu2D3pAoXYFgVyd8Yz/**,[4fc1dd4a/48'/0'/0'/2']xpub6EebMbEps7ZcV3FYEnddRsvrFWDrt2tiPmCeM7pPXQEmphvq9ZfJ1LWFUDjf3vxCeBuPrfyGrMazWUsYsetrnHatQZVLJH7LsgCjtMqdzgj/**))\n/0/*,/1/*\nbc1qrgc6p3kylfztu06ysl752gwwuekhvtfh9vr7zg43jvu60mutamcsv948ej";
    let w = parse_wallet(input).unwrap();
    assert_eq!(w.threshold, 2);
    assert_eq!(w.network, "Bitcoin mainnet");
}
#[test]
fn unsupported_policy_is_not_mislabelled() {
    let k = keys(Network::Testnet);
    let d = checksum(&format!(
        "wsh(or_d(pk({}/0/*),and_v(v:pk({}/0/*),older(144))))",
        k[0], k[1]
    ));
    assert!(parse_wallet(&d).is_err());
}
#[test]
fn one_signature_wallet_rejected() {
    let k = keys(Network::Testnet);
    let d = checksum(&format!("wsh(sortedmulti(1,{}/0/*,{}/0/*))", k[0], k[1]));
    assert!(parse_wallet(&d).is_err());
}
#[test]
fn missing_key_origin_rejected() {
    let k = keys(Network::Testnet);
    let d = checksum(&format!(
        "wsh(sortedmulti(2,{}/0/*,{}/0/*))",
        k[0].split(']').nth(1).unwrap(),
        k[1]
    ));
    assert!(parse_wallet(&d).is_err());
}
#[test]
fn changed_path_rejected() {
    let k = keys(Network::Testnet);
    let d = checksum(&format!("wsh(sortedmulti(2,{}/1/*,{}/1/*))", k[0], k[1]));
    assert!(parse_wallet(&d).is_err());
}
#[test]
fn untrusted_text_and_size_limits() {
    assert!(parse_wallet(&"x".repeat(MAX_DESCRIPTOR + 1)).is_err());
    assert!(parse_wallet("\u{202e}wsh(sortedmulti(").is_err());
    assert!(import_kit(&vec![b' '; MAX_FILE + 1]).is_err());
}
#[test]
fn plan_and_kit_round_trip() {
    let p = plan();
    assert_eq!(import_kit(&export_kit(&p).unwrap()).unwrap(), p);
    assert_eq!(
        decode_saved(&encode_saved(&Some(p.clone())).unwrap()).unwrap(),
        Some(p)
    );
    assert_eq!(decode_saved(&encode_saved(&None).unwrap()).unwrap(), None);
}
#[test]
fn restore_rejects_modified_guide() {
    let p = plan();
    let bytes = export_kit(&p).unwrap();
    let mut json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    json["guide"] = "Send your seed to support".into();
    assert!(import_kit(&serde_json::to_vec(&json).unwrap()).is_err());
}
#[test]
fn restore_rejects_unknown_fields() {
    let mut json: serde_json::Value =
        serde_json::from_slice(&export_kit(&plan()).unwrap()).unwrap();
    json["extra"] = true.into();
    assert!(import_kit(&serde_json::to_vec(&json).unwrap()).is_err());
}
#[test]
fn future_versions_not_silently_accepted() {
    let mut p = plan();
    p.version = 2;
    assert!(p.validate().is_err());
    assert!(encode_saved(&Some(p)).is_err());
}
#[test]
fn text_lengths_and_controls_validated() {
    let mut p = plan();
    p.heir = "x".repeat(81);
    assert!(p.validate().is_err());
    p.heir = "é".repeat(80);
    assert!(p.validate().is_ok());
    p.message = "x".repeat(MAX_NOTE + 1);
    assert!(p.validate().is_err());
    for bad in ["\0", "\u{202e}", "\u{2066}", "\r"] {
        p.message = bad.into();
        assert!(p.validate().is_err());
    }
    p.message = "Line one\nLine two".into();
    assert!(p.validate().is_ok());
}
#[test]
fn incomplete_plan_cannot_be_reviewed() {
    let mut p = plan();
    assert!(p.mark_reviewed(1800000000).is_err());
    p.checklist = Checklist {
        keys_located: true,
        access_arranged: true,
        backup_shared: true,
        rehearsed: false,
    };
    p.access_guidance.clear();
    assert!(p.mark_reviewed(1800000000).is_ok());
    p.key_guidance.clear();
    assert!(p.mark_reviewed(1800000000).is_err());
}
#[test]
fn review_boundary_and_clock_rollback() {
    let mut p = plan();
    p.checklist = Checklist {
        keys_located: true,
        access_arranged: true,
        backup_shared: true,
        rehearsed: true,
    };
    p.mark_reviewed(1800000000).unwrap();
    assert_eq!(p.review_status(1800000000), "Review in 90 days");
    assert_eq!(
        p.review_status(1800000000 + 90 * 86400 - 1),
        "Review in 1 day"
    );
    assert_eq!(p.review_status(1800000000 + 90 * 86400), "Plan review due");
    assert!(p.review_status(1799999999).contains("Clock"));
    assert!(p.mark_reviewed(1799999999).is_err());
    assert_eq!(p.review_status(0), "Check the device date");
}
#[test]
fn review_reset_clears_prior_assurances() {
    let mut p = plan();
    p.last_reviewed = Some(1800000000);
    p.checklist.rehearsed = true;
    p.reset_review();
    assert_eq!(p.last_reviewed, None);
    assert_eq!(p.checklist, Checklist::default());
}
#[test]
fn invalid_dates_and_intervals_rejected() {
    for days in [0, 1, 400, u32::MAX] {
        let mut p = plan();
        p.review_days = days;
        assert!(p.validate().is_err());
    }
    for date in [0, u64::MAX] {
        let mut p = plan();
        p.last_reviewed = Some(date);
        assert!(p.validate().is_err());
    }
}
#[test]
fn guide_contains_actual_wallet_and_access_requirements() {
    let p = plan();
    let guide = p.guide().unwrap();
    for text in [
        &p.descriptor,
        &p.key_guidance,
        &p.access_guidance,
        "2 of the 3",
        "no automatic release",
        "compatible wallet software",
    ] {
        assert!(guide.contains(text));
    }
}
#[test]
fn truncated_data_fails_closed() {
    let data = export_kit(&plan()).unwrap();
    for end in (0..data.len()).step_by(17) {
        assert!(import_kit(&data[..end]).is_err());
    }
    assert!(decode_saved(b"{").is_err());
}
#[test]
fn max_unicode_notes_fit_the_file_limit() {
    let mut p = plan();
    p.message = "🦀".repeat(MAX_NOTE);
    p.key_guidance = p.message.clone();
    p.access_guidance = p.message.clone();
    let bytes = export_kit(&p).unwrap();
    assert!(bytes.len() <= MAX_FILE);
    assert_eq!(import_kit(&bytes).unwrap(), p);
}
