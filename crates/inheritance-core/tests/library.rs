mod support;
use inheritance_core::*;
use support::plan;

#[test]
fn migrate_legacy_plan_preserves_every_field() {
    let p = plan();
    let old = encode_saved(&Some(p.clone())).unwrap();
    let library = decode_library(&old).unwrap();
    assert_eq!(library.entries.len(), 1);
    assert_eq!(library.entries[0].plan, p);
    assert_eq!(
        decode_library(&encode_library(&library).unwrap()).unwrap(),
        library
    );
}
#[test]
fn migrate_deleted_legacy_plan_stays_empty() {
    assert!(decode_library(b"null").unwrap().entries.is_empty());
}
#[test]
fn multiple_plans_roundtrip() {
    let mut library = Library::default();
    let first = library.put(None, plan()).unwrap();
    let mut second = plan();
    second.wallet_name = "Business".into();
    second.heir = "Sam".into();
    let id = library.put(None, second).unwrap();
    assert_ne!(first, id);
    assert_eq!(
        decode_library(&encode_library(&library).unwrap()).unwrap(),
        library
    );
}
#[test]
fn edit_one_plan_leaves_other_unchanged() {
    let mut library = Library::default();
    let a = library.put(None, plan()).unwrap();
    let b = library.put(None, plan()).unwrap();
    let untouched = library.entries[1].clone();
    let mut edited = plan();
    edited.heir = "Someone else".into();
    library.put(Some(a), edited).unwrap();
    assert_eq!(library.entries[1], untouched);
    assert_eq!(library.entries[1].id, b);
}
#[test]
fn deleting_one_does_not_delete_or_renumber_others() {
    let mut library = Library::default();
    let a = library.put(None, plan()).unwrap();
    let b = library.put(None, plan()).unwrap();
    library.remove(a).unwrap();
    assert_eq!(library.entries[0].id, b);
    assert!(library.put(None, plan()).unwrap() > b);
    assert!(library.remove(a).is_err());
}
#[test]
fn restore_adds_a_plan_without_replacing_existing() {
    let mut library = Library::default();
    library.put(None, plan()).unwrap();
    let before = library.entries[0].clone();
    let mut restored = import_kit(&export_kit(&plan()).unwrap()).unwrap();
    restored.reset_review();
    library.put(None, restored).unwrap();
    assert_eq!(library.entries.len(), 2);
    assert_eq!(library.entries[0], before);
}
#[test]
fn failed_insert_preserves_collection() {
    let mut library = Library::default();
    library.put(None, plan()).unwrap();
    let before = library.clone();
    let mut invalid = plan();
    invalid.descriptor = "invalid".into();
    assert!(library.put(None, invalid).is_err());
    assert_eq!(library, before);
    assert!(library.put(Some(999), plan()).is_err());
    assert_eq!(library, before);
}
#[test]
fn capacity_is_bounded_but_existing_plans_remain_editable() {
    let mut library = Library::default();
    for _ in 0..MAX_PLANS {
        library.put(None, plan()).unwrap();
    }
    let before = library.clone();
    assert!(library.put(None, plan()).is_err());
    assert_eq!(library, before);
    assert!(library.put(Some(1), plan()).is_ok());
    assert!(encode_library(&library).is_ok());
}
#[test]
fn corrupt_collection_does_not_fall_back_to_empty() {
    for bytes in [
        b"{}".as_slice(),
        b"[]",
        b"{\"version\":3}",
        b"{\"version\":2}",
        b"{\"version\":1}",
    ] {
        assert!(decode_library(bytes).is_err());
    }
}
#[test]
fn duplicate_or_invalid_ids_rejected() {
    let mut library = Library::default();
    library.put(None, plan()).unwrap();
    library.put(None, plan()).unwrap();
    for id in [0, 1, 999] {
        let mut invalid = library.clone();
        invalid.entries[1].id = id;
        assert!(encode_library(&invalid).is_err());
    }
}
#[test]
fn independent_review_state() {
    let mut library = Library::default();
    let a = library.put(None, plan()).unwrap();
    library.put(None, plan()).unwrap();
    let mut reviewed = plan();
    reviewed.checklist = Checklist {
        keys_located: true,
        access_arranged: true,
        backup_shared: true,
        rehearsed: true,
    };
    reviewed.mark_reviewed(1788849000).unwrap();
    library.put(Some(a), reviewed).unwrap();
    assert!(library.entries[0].plan.last_reviewed.is_some());
    assert!(library.entries[1].plan.last_reviewed.is_none());
}
#[test]
fn library_with_maximum_notes_fits_storage_limit() {
    let mut library = Library::default();
    let mut p = plan();
    p.message = "🌱".repeat(MAX_NOTE);
    p.key_guidance = p.message.clone();
    p.access_guidance = p.message.clone();
    for _ in 0..MAX_PLANS {
        library.put(None, p.clone()).unwrap();
    }
    assert_eq!(
        decode_library(&encode_library(&library).unwrap()).unwrap(),
        library
    );
}
