mod support;
use inheritance_core::*;
const NOW: u64 = 1788849000;

fn complete() -> Plan {
    let mut p = support::plan();
    p.upgrade().unwrap();
    for (i, s) in p.care.as_mut().unwrap().signers.iter_mut().enumerate() {
        s.name = format!("Key {}", i + 1);
        s.location = "Home safe".into();
        s.access = "Ask the executor for the separate instructions".into();
    }
    p
}

#[test]
fn handover_and_practice_migrate_and_reset_safely() {
    let mut p = complete();
    let mut old = serde_json::to_value(&p).unwrap();
    old["care"].as_object_mut().unwrap().remove("handover");
    old["care"].as_object_mut().unwrap().remove("practice");
    let migrated: Plan = serde_json::from_value(old).unwrap();
    assert_eq!(migrated.care.as_ref().unwrap().handover, [false; 4]);
    assert!(migrated.care.as_ref().unwrap().practice.is_none());
    p.care.as_mut().unwrap().handover = [true; 4];
    p.record_practice([true, false, true, false, false], NOW)
        .unwrap();
    assert!(!p.checklist.rehearsed);
    assert!(p.care.as_ref().unwrap().rehearsed_at.is_none());
    assert_eq!(
        p.care.as_ref().unwrap().practice.as_ref().unwrap().checks,
        [true, false, true, false, false]
    );
    p.record_practice([true; 5], NOW + 1).unwrap();
    assert!(p.checklist.rehearsed);
    assert!(p.last_reviewed.is_none());
    assert!(p.record_practice([true; 5], NOW).is_err());
    p.forget_imported_assurances();
    assert_eq!(p.care.as_ref().unwrap().handover, [false; 4]);
    assert!(p.care.as_ref().unwrap().practice.is_none());
}

#[test]
fn incomplete_practice_is_not_success_and_custodian_references_work() {
    let mut p = complete();
    for s in &mut p.care.as_mut().unwrap().signers {
        s.location = "Contact the executor".into();
        s.access = "Executor holds separate access instructions".into();
    }
    assert!(p.completeness().is_empty());
    p.record_practice([true; 5], NOW).unwrap();
    p.record_practice([false; 5], NOW + 1).unwrap();
    assert!(!p.checklist.rehearsed);
    assert!(p.care.as_ref().unwrap().rehearsed_at.is_none());
    p.heir.clear();
    assert!(p.record_practice([true; 5], NOW + 2).is_err());
    p.record_practice([false; 5], NOW + 2).unwrap();
    let guide = p.recovery_guide().unwrap();
    assert!(!guide.contains("[ ] Rehearse with the owner"));
    assert!(guide.contains("Owner preparation (before handover)"));
    assert!(guide.contains("outside this locked plan"));
    assert!(guide.contains("Decrypted files are plaintext"));
}

#[test]
fn review_and_practice_keep_independent_dates() {
    let mut p = complete();
    p.checklist.keys_located = true;
    p.checklist.access_arranged = true;
    p.checklist.backup_shared = true;
    p.mark_reviewed(NOW).unwrap();
    assert_eq!(p.care.as_ref().unwrap().rehearsed_at, None);
    assert!(!p.checklist.rehearsed);
    p.record_rehearsal(&[true; 3], NOW + 60).unwrap();
    assert_eq!(p.last_reviewed, Some(NOW));
    p.mark_reviewed(NOW + 120).unwrap();
    assert_eq!(p.care.as_ref().unwrap().rehearsed_at, Some(NOW + 60));
    assert!(p.checklist.rehearsed);
    assert_eq!(p.last_reviewed, Some(NOW + 120));
}

#[test]
fn upgrade_retains_legacy_notes_and_identity() {
    let mut p = support::plan();
    let before = p.clone();
    p.upgrade().unwrap();
    assert_eq!(p.key_guidance, before.key_guidance);
    assert_eq!(p.care.as_ref().unwrap().signers.len(), 3);
    assert!(p.same_wallet(&before));
    assert!(!p.completeness().is_empty());
    let upgraded = p.clone();
    p.upgrade().unwrap();
    assert_eq!(p, upgraded);
}
#[test]
fn completeness_counts_distinct_described_keys_only() {
    let mut p = complete();
    assert!(p.completeness().is_empty());
    p.care.as_mut().unwrap().signers[2].access.clear();
    assert!(p.completeness().is_empty());
    p.care.as_mut().unwrap().signers[1].location.clear();
    assert!(p.completeness().join(" ").contains("only 1"));
}
#[test]
fn unknown_duplicate_and_missing_signers_rejected() {
    for kind in 0..3 {
        let mut p = complete();
        let c = p.care.as_mut().unwrap();
        match kind {
            0 => c.signers[0].fingerprint = "DEADBEEF".into(),
            1 => c.signers[0].fingerprint = c.signers[1].fingerprint.clone(),
            _ => {
                c.signers.pop();
            }
        }
        assert!(p.validate().is_err());
    }
}
#[test]
fn signer_limits_and_controls_rejected() {
    for text in [
        "a".repeat(241),
        "bad\u{202e}text".into(),
        "bad\0text".into(),
    ] {
        let mut p = complete();
        p.care.as_mut().unwrap().signers[0].access = text;
        assert!(p.validate().is_err());
    }
}
#[test]
fn revision_changes_and_noop_is_stable() {
    let mut p = complete();
    p.revise(None, NOW).unwrap();
    let old = p.clone();
    p.revise(Some(&old), NOW + 1).unwrap();
    assert_eq!(p, old);
    p.heir = "New heir".into();
    p.revise(Some(&old), NOW + 1).unwrap();
    assert_eq!(p.care.as_ref().unwrap().revision, 2);
    assert_eq!(p.care.as_ref().unwrap().modified_at, Some(NOW + 1));
}
#[test]
fn export_receipt_does_not_change_revision_and_edit_stales_it() {
    let mut p = complete();
    p.revise(None, NOW).unwrap();
    let old = p.clone();
    p.care.as_mut().unwrap().exported_revision = Some(1);
    p.care.as_mut().unwrap().encrypted_export = true;
    p.revise(Some(&old), NOW).unwrap();
    assert_eq!(p.care.as_ref().unwrap().revision, 1);
    assert!(p.export_status().contains("Revision 1 backed up"));
    let exported = p.clone();
    p.message += " Changed";
    p.revise(Some(&exported), NOW + 1).unwrap();
    assert!(p.export_status().contains("out of date"));
}
#[test]
fn corrupt_revision_and_dates_rejected() {
    for kind in 0..4 {
        let mut p = complete();
        let c = p.care.as_mut().unwrap();
        match kind {
            0 => c.revision = 0,
            1 => c.exported_revision = Some(2),
            2 => c.modified_at = Some(1),
            _ => c.rehearsed_at = Some(u64::MAX),
        };
        assert!(p.validate().is_err());
    }
}
#[test]
fn revision_overflow_returns_error() {
    let mut p = complete();
    p.care.as_mut().unwrap().revision = u64::MAX;
    let old = p.clone();
    p.message += "changed";
    assert!(p.revise(Some(&old), NOW).is_err());
}
#[test]
fn new_kit_roundtrips_with_actionable_guide() {
    let mut p = complete();
    p.revise(None, NOW).unwrap();
    let kit = export_kit(&p).unwrap();
    assert_eq!(import_kit(&kit).unwrap(), p);
    let guide = p.guide().unwrap();
    for text in [
        "Revision: 1",
        "2026-09-08",
        "Location: Home safe",
        "Access: Ask",
        "[ ]",
        "master seed",
        "not an inheritance-service",
    ] {
        assert!(guide.contains(text), "{text}");
    }
}
#[test]
fn mismatched_kit_schema_or_guide_rejected() {
    let bytes = export_kit(&complete()).unwrap();
    let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    v["format"] = "passport-inheritance-kit-v1".into();
    assert!(import_kit(&serde_json::to_vec(&v).unwrap()).is_err());
    v["format"] = "passport-inheritance-kit-v2".into();
    v["guide"] = "wrong".into();
    assert!(import_kit(&serde_json::to_vec(&v).unwrap()).is_err());
}
#[test]
fn restoring_forgets_export_and_rehearsal_assurances() {
    let mut p = complete();
    p.record_rehearsal(&[true; 3], NOW).unwrap();
    p.care.as_mut().unwrap().exported_revision = Some(1);
    let mut restored = import_kit(&export_kit(&p).unwrap()).unwrap();
    restored.forget_imported_assurances();
    let c = restored.care.as_ref().unwrap();
    assert_eq!(c.exported_revision, None);
    assert_eq!(c.rehearsed_at, None);
    assert!(!restored.checklist.rehearsed);
}
#[test]
fn rehearsal_requires_all_steps_and_details() {
    let mut p = complete();
    for checks in [vec![], vec![true], vec![true, false, true], vec![true; 4]] {
        assert!(p.record_rehearsal(&checks, NOW).is_err());
    }
    p.heir.clear();
    assert!(p.record_rehearsal(&[true; 3], NOW).is_err());
}
#[test]
fn rehearsal_date_recorded_and_edit_reset() {
    let mut p = complete();
    p.record_rehearsal(&[true; 3], NOW).unwrap();
    assert_eq!(p.care.as_ref().unwrap().rehearsed_at, Some(NOW));
    assert!(p.record_rehearsal(&[true; 3], NOW - 1).is_err());
    p.reset_review();
    assert_eq!(p.care.as_ref().unwrap().rehearsed_at, None);
}

#[test]
fn rehearsal_errors_identify_the_actual_blocker() {
    let mut p = complete();
    assert_eq!(
        p.record_rehearsal(&[false; 3], NOW).unwrap_err(),
        "Confirm all three rehearsal steps before recording."
    );
    p.heir.clear();
    let error = p.record_rehearsal(&[true; 3], NOW).unwrap_err();
    assert!(error.contains("Add heir."));
    assert!(!error.contains("device date"));
    assert_eq!(p.care.as_ref().unwrap().rehearsed_at, None);
    p.heir = "Test heir".into();
    assert_eq!(
        p.record_rehearsal(&[true; 3], 0).unwrap_err(),
        "Check the device date before recording the rehearsal."
    );
    p.record_rehearsal(&[true; 3], NOW).unwrap();
}
#[test]
fn wallet_identity_ignores_plan_name_but_not_wallet() {
    let p = complete();
    let mut other = p.clone();
    other.wallet_name = "Other plan".into();
    assert!(p.same_wallet(&other));
    assert_eq!(p.wallet_id().unwrap(), other.wallet_id().unwrap());
    other.descriptor = "bad".into();
    assert!(!p.same_wallet(&other));
}

#[test]
fn additional_instructions_are_optional_for_review_and_rehearsal() {
    let mut p = complete();
    p.access_guidance.clear();
    assert!(p.completeness().is_empty());
    assert!(p.has_guidance());
    p.checklist.keys_located = true;
    p.checklist.access_arranged = true;
    p.checklist.backup_shared = true;
    p.mark_reviewed(NOW).unwrap();
    p.record_rehearsal(&[true; 3], NOW).unwrap();
    p.access_guidance = "Existing extra instructions".into();
    let backup = encrypt_backup(&p, "separate river lantern orchard").unwrap();
    let restored = decrypt_backup(&backup, "separate river lantern orchard").unwrap();
    assert_eq!(restored.access_guidance, p.access_guidance);
    assert!(restored
        .guide()
        .unwrap()
        .contains("Existing extra instructions"));
    for signer in &mut p.care.as_mut().unwrap().signers {
        signer.access.clear();
    }
    assert!(!p.completeness().is_empty());
    assert!(p.record_rehearsal(&[true; 3], NOW).is_err());
}
#[test]
fn maximum_unicode_signer_notes_and_legacy_notes_fit() {
    let mut p = complete();
    p.message = "🌱".repeat(MAX_NOTE);
    p.key_guidance = p.message.clone();
    p.access_guidance = p.message.clone();
    for s in &mut p.care.as_mut().unwrap().signers {
        s.name = "🌱".repeat(80);
        s.location = "🌱".repeat(240);
        s.access = s.location.clone();
    }
    assert!(export_kit(&p).unwrap().len() <= MAX_FILE);
    let mut library = Library::default();
    for _ in 0..MAX_PLANS {
        library.put(None, p.clone()).unwrap();
    }
    assert_eq!(
        decode_library(&encode_library(&library).unwrap()).unwrap(),
        library
    );
}
#[test]
fn date_formatting_is_bounded_and_utc() {
    assert_eq!(utc_date(Some(1709164800)), "2024-02-29");
    assert_eq!(utc_date(Some(u64::MAX)), "Check device date");
    assert_eq!(utc_date(None), "Not recorded");
}

#[test]
fn five_signers_at_all_text_limits_export_and_restore() {
    use miniscript::bitcoin::{
        bip32::{Xpriv, Xpub},
        secp256k1::Secp256k1,
        Network,
    };
    let keys: Vec<_> = (1..=5)
        .map(|i| {
            let root = Xpriv::new_master(Network::Testnet, &[i; 32]).unwrap();
            let key = Xpub::from_priv(&Secp256k1::new(), &root);
            format!("[{}]{key}/<0;1>/*", key.fingerprint())
        })
        .collect();
    let descriptor = support::checksum(&format!("wsh(sortedmulti(3,{}))", keys.join(",")));
    let mut p = Plan::new(&parse_wallet(&descriptor).unwrap());
    p.upgrade().unwrap();
    p.wallet_name = "🌱".repeat(80);
    p.heir = p.wallet_name.clone();
    p.contact = "🌱".repeat(200);
    p.message = "🌱".repeat(MAX_NOTE);
    p.key_guidance = p.message.clone();
    p.access_guidance = p.message.clone();
    for s in &mut p.care.as_mut().unwrap().signers {
        s.name = "🌱".repeat(80);
        s.location = "🌱".repeat(240);
        s.access = s.location.clone();
    }
    let bytes = export_kit(&p).unwrap();
    assert_eq!(import_kit(&bytes).unwrap(), p);
    let mut library = Library::default();
    for _ in 0..MAX_PLANS {
        library.put(None, p.clone()).unwrap();
    }
    assert!(encode_library(&library).is_ok());
}
