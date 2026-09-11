use crate::{Plan, MAX_FILE};
use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{
    aead::{Aead, Payload},
    KeyInit, XChaCha20Poly1305, XNonce,
};
use miniscript::bitcoin::hashes::{sha256, Hash};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read, Write};
use zeroize::Zeroizing;
use zip::{write::SimpleFileOptions, AesMode, CompressionMethod, ZipArchive, ZipWriter};

const MAGIC: &[u8; 8] = b"INHENC01";
const HEADER: usize = 48;
pub const MAX_BACKUP: usize = 3 * MAX_FILE + 4096;
const MEMBERS: [&str; 3] = ["recovery-guide.txt", "wallet.bsms", "plan.json"];

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArchivePlan {
    format: u32,
    plan: Plan,
    guide_sha256: String,
    wallet_sha256: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackupPlan {
    format: u32,
    plan: Plan,
}

pub fn validate_backup_password(password: &str) -> Result<(), String> {
    if password.chars().count() < 20
        || password.len() > 256
        || password.chars().all(char::is_whitespace)
        || password.chars().any(char::is_control)
    {
        return Err("Use at least 20 characters (up to 256 bytes). Choose five or more randomly chosen words, not your device PIN.".into());
    }
    Ok(())
}

pub fn validate_encrypted_backup(bytes: &[u8]) -> Result<(), String> {
    if bytes.starts_with(MAGIC) && (HEADER + 16..=MAX_FILE + HEADER + 16).contains(&bytes.len()) {
        return Ok(());
    }
    if bytes.len() > MAX_BACKUP || !bytes.starts_with(b"PK\x03\x04") {
        return Err(
            "Choose an encrypted recovery ZIP or older .inheritance backup. Plaintext backups are not accepted."
                .into(),
        );
    }
    // Bound metadata parsing to a small, comment-free, non-ZIP64 archive.
    if bytes.len() < 22
        || &bytes[bytes.len() - 22..bytes.len() - 18] != b"PK\x05\x06"
        || bytes[bytes.len() - 2..] != [0, 0]
    {
        return Err("Unsupported recovery ZIP structure.".into());
    }
    let end = &bytes[bytes.len() - 22..];
    if end[4..12] != [0, 0, 0, 0, 3, 0, 3, 0]
        || u64::from(u32::from_le_bytes(end[12..16].try_into().unwrap()))
            + u64::from(u32::from_le_bytes(end[16..20].try_into().unwrap()))
            != (bytes.len() - 22) as u64
    {
        return Err("Unsupported recovery ZIP directory.".into());
    }
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|_| "Invalid recovery ZIP.")?;
    if archive.len() != MEMBERS.len() || archive.offset() != 0 {
        return Err("The recovery ZIP must contain exactly three recovery files.".into());
    }
    for (index, name) in MEMBERS.iter().enumerate() {
        let file = archive
            .by_index_raw(index)
            .map_err(|_| "Invalid ZIP member.")?;
        if file.name() != *name
            || !file.encrypted()
            || file.compression() != CompressionMethod::Stored
            || file.size() > MAX_FILE as u64
            || file.compressed_size() != file.size() + 28
        {
            return Err("Recovery ZIP files must all use AES-256 encryption and bounded, uncompressed contents.".into());
        }
        let extra = file
            .extra_data()
            .ok_or("Missing ZIP encryption information.")?;
        let mut fields = extra;
        let mut aes = false;
        while !fields.is_empty() {
            if fields.len() < 4 {
                return Err("Invalid ZIP metadata.".into());
            }
            let id = u16::from_le_bytes([fields[0], fields[1]]);
            if id == 1 {
                return Err("ZIP64 recovery archives are not supported.".into());
            }
            let len = u16::from_le_bytes([fields[2], fields[3]]) as usize;
            if fields.len() < 4 + len {
                return Err("Invalid ZIP metadata.".into());
            }
            if id == 0x9901 {
                if aes
                    || len != 7
                    || ![1, 2].contains(&fields[4])
                    || fields[5..11] != [0, b'A', b'E', 3, 0, 0]
                {
                    return Err("Only AES-256 recovery ZIPs are supported.".into());
                }
                aes = true;
            }
            fields = &fields[4 + len..];
        }
        if !aes {
            return Err("Legacy ZIP encryption is not supported.".into());
        }
    }
    Ok(())
}

fn key(password: &str, salt: &[u8]) -> Result<Zeroizing<[u8; 32]>, String> {
    if password.len() > 256 {
        return Err("Password is too long.".into());
    }
    // Version 1 fixes KDF costs so untrusted files cannot request unbounded work.
    let params =
        Params::new(19 * 1024, 2, 1, Some(32)).map_err(|_| "Password protection unavailable.")?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = Zeroizing::new([0u8; 32]);
    argon
        .hash_password_into(password.as_bytes(), salt, key.as_mut())
        .map_err(|_| "Could not derive the backup key.")?;
    Ok(key)
}

/// Encrypt a plan using fresh operating-system randomness for every backup.
pub fn encrypt_backup(plan: &Plan, password: &str) -> Result<Vec<u8>, String> {
    validate_backup_password(password)?;
    plan.validate()?;
    let guide = Zeroizing::new(plan.recovery_guide()?.into_bytes());
    let wallet = Zeroizing::new(plan.bsms()?.into_bytes());
    let data = Zeroizing::new(
        serde_json::to_vec(&ArchivePlan {
            format: 2,
            plan: plan.clone(),
            guide_sha256: sha256::Hash::hash(&guide).to_string(),
            wallet_sha256: sha256::Hash::hash(&wallet).to_string(),
        })
        .map_err(|_| "Could not encode backup.")?,
    );
    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, contents) in MEMBERS.iter().zip([&guide, &wallet, &data]) {
        if contents.len() > MAX_FILE {
            return Err("Backup exceeds the size limit.".into());
        }
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Stored)
            .with_aes_encryption(AesMode::Aes256, password);
        archive
            .start_file(*name, options)
            .map_err(|_| "Could not encrypt the recovery ZIP.")?;
        archive
            .write_all(contents)
            .map_err(|_| "Could not encrypt the recovery ZIP.")?;
    }
    let output = archive
        .finish()
        .map_err(|_| "Could not finish the recovery ZIP.")?
        .into_inner();
    validate_encrypted_backup(&output)?;
    Ok(output)
}

/// Authenticate before parsing; never fall back to plaintext on failure.
pub fn decrypt_backup(bytes: &[u8], password: &str) -> Result<Plan, String> {
    validate_encrypted_backup(bytes)?;
    if !bytes.starts_with(MAGIC) {
        return decrypt_zip(bytes, password);
    }
    let key = key(password, &bytes[8..24])?;
    let cipher = XChaCha20Poly1305::new_from_slice(key.as_ref())
        .map_err(|_| "Could not initialise decryption.")?;
    let plaintext = Zeroizing::new(
        cipher
            .decrypt(
                XNonce::from_slice(&bytes[24..HEADER]),
                Payload {
                    msg: &bytes[HEADER..],
                    aad: &bytes[..HEADER],
                },
            )
            .map_err(|_| "Incorrect password or damaged backup. Nothing was restored.")?,
    );
    let restored: BackupPlan =
        serde_json::from_slice(&plaintext).map_err(|_| "Invalid encrypted plan data.")?;
    if restored.format != 1 {
        return Err("This backup version is not supported.".into());
    }
    restored.plan.validate()?;
    Ok(restored.plan)
}

fn decrypt_zip(bytes: &[u8], password: &str) -> Result<Plan, String> {
    if password.len() > 256 {
        return Err("Password is too long.".into());
    }
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|_| "Invalid recovery ZIP.")?;
    let mut contents = Vec::new();
    for index in 0..MEMBERS.len() {
        let file = archive
            .by_index_decrypt(index, password.as_bytes())
            .map_err(|_| "Incorrect password or damaged backup. Nothing was restored.")?;
        let mut data = Zeroizing::new(Vec::new());
        file.take((MAX_FILE + 1) as u64)
            .read_to_end(&mut data)
            .map_err(|_| "Incorrect password or damaged backup. Nothing was restored.")?;
        if data.len() > MAX_FILE {
            return Err("Backup exceeds the size limit.".into());
        }
        contents.push(data);
    }
    let restored: ArchivePlan =
        serde_json::from_slice(&contents[2]).map_err(|_| "Invalid encrypted plan data.")?;
    if restored.format != 2
        || restored.guide_sha256 != sha256::Hash::hash(&contents[0]).to_string()
        || restored.wallet_sha256 != sha256::Hash::hash(&contents[1]).to_string()
    {
        return Err("The recovery files do not belong to the same backup.".into());
    }
    restored.plan.validate()?;
    if restored.plan.bsms()?.as_bytes() != contents[1].as_slice() {
        return Err("Wallet configuration does not match the saved plan.".into());
    }
    Ok(restored.plan)
}
