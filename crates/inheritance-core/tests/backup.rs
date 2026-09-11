mod support;
use inheritance_core::*;
const PASSWORD: &str = "separate river lantern orchard";

#[test]
fn encrypted_roundtrip_and_fresh_randomness() {
    let p = support::plan();
    let a = encrypt_backup(&p, PASSWORD).unwrap();
    let b = encrypt_backup(&p, PASSWORD).unwrap();
    assert!(a.starts_with(b"PK\x03\x04"));
    assert_ne!(a, b);
    assert_eq!(decrypt_backup(&a, PASSWORD).unwrap(), p);
    for secret in [&p.wallet_name, &p.descriptor, &p.contact, &p.key_guidance] {
        assert!(!a.windows(secret.len()).any(|w| w == secret.as_bytes()));
    }
}

#[test]
fn wrong_password_and_tampering_fail_closed() {
    let a = encrypt_backup(&support::plan(), PASSWORD).unwrap();
    assert!(decrypt_backup(&a, "wrong password").is_err());
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&a)).unwrap();
    let start = archive.by_index_raw(0).unwrap().data_start() as usize;
    for index in [0, start, start + 16, start + 18, a.len() - 1] {
        let mut damaged = a.clone();
        damaged[index] ^= 1;
        assert!(decrypt_backup(&damaged, PASSWORD).is_err());
    }
    let mut extra = a.clone();
    extra.push(0);
    assert!(decrypt_backup(&extra, PASSWORD).is_err());
    assert!(decrypt_backup(&a[..a.len() - 1], PASSWORD).is_err());
}

#[test]
fn plaintext_unknown_versions_and_limits_rejected() {
    assert!(decrypt_backup(&export_kit(&support::plan()).unwrap(), PASSWORD).is_err());
    for n in [0, 8, 47, 63, MAX_BACKUP + 1] {
        assert!(validate_encrypted_backup(&vec![0; n]).is_err());
    }
    assert!(validate_encrypted_backup(b"INHENC02").is_err());
}

#[test]
fn password_policy() {
    for p in ["", "1234", "           ", "twelve chars\n"] {
        assert!(validate_backup_password(p).is_err());
    }
    assert!(validate_backup_password(&"a".repeat(257)).is_err());
    assert!(validate_backup_password(PASSWORD).is_ok());
    assert!(validate_backup_password("étoile rivière arbre matin").is_ok());
}

#[test]
fn legacy_export_not_mistaken_for_encrypted_backup() {
    let mut p = support::plan();
    p.upgrade().unwrap();
    p.care.as_mut().unwrap().exported_revision = Some(1);
    assert!(p.export_status().contains("plaintext"));
}

#[test]
fn zip_members_are_individually_encrypted_and_authenticated() {
    use std::io::{Cursor, Read};
    let bytes = encrypt_backup(&support::plan(), PASSWORD).unwrap();
    let mut archive = zip::ZipArchive::new(Cursor::new(&bytes)).unwrap();
    for index in 0..3 {
        let (start, end) = {
            let file = archive.by_index_raw(index).unwrap();
            assert!(file.encrypted());
            (
                file.data_start() as usize,
                (file.data_start() + file.compressed_size()) as usize,
            )
        };
        assert!(archive.by_index(index).is_err());
        let mut plain = Vec::new();
        archive
            .by_index_decrypt(index, PASSWORD.as_bytes())
            .unwrap()
            .read_to_end(&mut plain)
            .unwrap();
        assert!(!plain.is_empty());
        for position in [start, start + 17, start + 19, end - 1] {
            let mut changed = bytes.clone();
            changed[position] ^= 1;
            assert!(decrypt_backup(&changed, PASSWORD).is_err());
        }
    }
}

#[test]
fn zip_rejects_plaintext_extra_members_and_path_traversal() {
    use std::io::{Cursor, Write};
    for names in [
        vec!["recovery-guide.txt", "wallet.bsms", "plan.json"],
        vec!["../plan.json"],
        vec!["plan.json", "extra.txt"],
    ] {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for name in names {
            writer
                .start_file(name, zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(b"not encrypted").unwrap();
        }
        let bytes = writer.finish().unwrap().into_inner();
        assert!(validate_encrypted_backup(&bytes).is_err());
    }
    let bytes = encrypt_backup(&support::plan(), PASSWORD).unwrap();
    for count in [0u16, 4, u16::MAX] {
        let mut changed = bytes.clone();
        let end = changed.len() - 22;
        changed[end + 8..end + 10].copy_from_slice(&count.to_le_bytes());
        changed[end + 10..end + 12].copy_from_slice(&count.to_le_bytes());
        assert!(validate_encrypted_backup(&changed).is_err());
    }
}

#[test]
fn older_inheritance_backup_still_restores() {
    use argon2::{Algorithm, Argon2, Params, Version};
    use chacha20poly1305::{
        aead::{Aead, Payload},
        KeyInit, XChaCha20Poly1305, XNonce,
    };
    let p = support::plan();
    let mut header = [0u8; 48];
    header[..8].copy_from_slice(b"INHENC01");
    let mut key = [0u8; 32];
    Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(19456, 2, 1, Some(32)).unwrap(),
    )
    .hash_password_into(PASSWORD.as_bytes(), &header[8..24], &mut key)
    .unwrap();
    let plain = serde_json::to_vec(&serde_json::json!({"format":1,"plan":p})).unwrap();
    let encrypted = XChaCha20Poly1305::new_from_slice(&key)
        .unwrap()
        .encrypt(
            XNonce::from_slice(&header[24..]),
            Payload {
                msg: &plain,
                aad: &header,
            },
        )
        .unwrap();
    let mut backup = header.to_vec();
    backup.extend_from_slice(&encrypted);
    assert_eq!(decrypt_backup(&backup, PASSWORD).unwrap(), p);
    assert!(decrypt_backup(&backup, "wrong password").is_err());
}
