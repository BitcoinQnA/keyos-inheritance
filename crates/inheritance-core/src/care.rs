use super::{parse_wallet, Plan, Wallet};
use miniscript::bitcoin::hashes::{sha256, Hash};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Signer {
    pub fingerprint: String,
    pub name: String,
    pub location: String,
    pub access: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Care {
    pub signers: Vec<Signer>,
    pub revision: u64,
    pub modified_at: Option<u64>,
    pub exported_revision: Option<u64>,
    #[serde(default)]
    pub encrypted_export: bool,
    pub rehearsed_at: Option<u64>,
    #[serde(default)]
    pub handover: [bool; 4],
    #[serde(default)]
    pub practice: Option<Practice>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Practice {
    pub at: u64,
    pub checks: [bool; 5],
}

fn valid_date(date: Option<u64>) -> bool {
    date.is_none_or(|t| (1577836800..=4102444800).contains(&t))
}

fn check_text(value: &str, max: usize) -> Result<(), String> {
    if value.chars().count() > max
        || value.chars().any(|c| {
            (c.is_control() && c != '\n')
                || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
    {
        // TODO: localize
        return Err(format!(
            "Use up to {max} characters without hidden control characters."
        ));
    }
    Ok(())
}

impl Care {
    pub fn validate(&self, wallet: &Wallet) -> Result<(), String> {
        let ids: BTreeSet<_> = self.signers.iter().map(|s| &s.fingerprint).collect();
        if self.revision == 0
            || self.signers.len() != wallet.fingerprints.len()
            || ids.len() != self.signers.len()
            || wallet.fingerprints.iter().any(|f| !ids.contains(f))
            || self
                .signers
                .iter()
                .zip(&wallet.fingerprints)
                .any(|(s, f)| &s.fingerprint != f)
            || self
                .exported_revision
                .is_some_and(|r| r == 0 || r > self.revision)
            || !valid_date(self.modified_at)
            || !valid_date(self.rehearsed_at)
            || self
                .practice
                .as_ref()
                .is_some_and(|p| !valid_date(Some(p.at)))
        {
            // TODO: localize
            return Err("Invalid signer identity, revision or date in this plan.".into());
        }
        for signer in &self.signers {
            check_text(&signer.name, 80)?;
            check_text(&signer.location, 240)?;
            check_text(&signer.access, 240)?;
        }
        Ok(())
    }
}

impl Plan {
    pub fn upgrade(&mut self) -> Result<(), String> {
        let wallet = self.validate()?;
        if self.care.is_none() {
            self.care = Some(Care {
                signers: wallet
                    .fingerprints
                    .into_iter()
                    .map(|fingerprint| Signer {
                        fingerprint,
                        name: String::new(),
                        location: String::new(),
                        access: String::new(),
                    })
                    .collect(),
                revision: 1,
                modified_at: None,
                exported_revision: None,
                encrypted_export: false,
                rehearsed_at: None,
                handover: [false; 4],
                practice: None,
            });
        }
        Ok(())
    }

    pub fn completeness(&self) -> Vec<String> {
        let mut missing = Vec::new();
        // TODO: localize
        for (name, value) in [
            ("plan name", &self.wallet_name),
            ("heir", &self.heir),
            ("trusted contact", &self.contact),
        ] {
            if value.trim().is_empty() {
                missing.push(format!("Add {name}."));
            }
        }
        if let Ok(wallet) = self.validate() {
            let count_for = |ids: &[String]| {
                self.care.as_ref().map_or(0, |c| {
                    c.signers
                        .iter()
                        .filter(|s| {
                            ids.contains(&s.fingerprint)
                                && !s.location.trim().is_empty()
                                && !s.access.trim().is_empty()
                        })
                        .count()
                })
            };
            let normal_count = wallet
                .inheritance
                .as_ref()
                .map_or(wallet.fingerprints.len(), |p| p.normal_count);
            let count = count_for(&wallet.fingerprints[..normal_count]);
            if count < wallet.threshold {
                // TODO: localize
                missing.push(format!("You need {} signing keys; only {count} have both location and access instructions.", wallet.threshold));
            }
            if let Some(p) = &wallet.inheritance {
                let count = count_for(&wallet.fingerprints[p.normal_count..]);
                if count < p.threshold {
                    missing.push(format!("Inheritance path needs {} inheritance keys; only {count} have location and access instructions. Normal keys do not count toward this path.", p.threshold));
                }
            }
        } else {
            // TODO: localize
            missing.push("Wallet details need attention.".into());
        }
        missing
    }

    pub fn signer_instructions(&self) -> String {
        let mut sections = Vec::new();
        let wallet = self.validate().ok();
        if let Some(w) = &wallet {
            if w.inheritance.is_some() {
                sections.push(w.signing_rules());
            }
        }
        if let Some(care) = &self.care {
            for (i, s) in care.signers.iter().enumerate() {
                if let Some(w) = &wallet {
                    if w.inheritance.is_some() {
                        sections.push(w.signer_role(i).into());
                    }
                }
                // TODO: localize
                sections.push(format!(
                    "{}. {} ({})\nLocation: {}\nAccess: {}",
                    i + 1,
                    if s.name.is_empty() {
                        wallet.as_ref().map_or("Signing key", |w| w.signer_role(i))
                    } else {
                        &s.name
                    },
                    s.fingerprint,
                    if s.location.is_empty() {
                        "Not recorded"
                    } else {
                        &s.location
                    },
                    if s.access.is_empty() {
                        "Not recorded"
                    } else {
                        &s.access
                    }
                ));
            }
        }
        if !self.key_guidance.trim().is_empty() {
            // TODO: localize
            sections.push(format!("Additional notes:\n{}", self.key_guidance));
        }
        sections.join("\n\n")
    }

    pub fn wallet_id(&self) -> Result<String, String> {
        let wallet = self.validate()?;
        Ok(sha256::Hash::hash(wallet.first_address.as_bytes()).to_string()[..8].to_uppercase())
    }

    pub fn same_wallet(&self, other: &Self) -> bool {
        match (
            parse_wallet(&self.descriptor),
            parse_wallet(&other.descriptor),
        ) {
            (Ok(a), Ok(b)) => a.first_address == b.first_address && a.network == b.network,
            _ => false,
        }
    }

    /// Prepare a persisted change; export receipts alone do not create revisions.
    pub fn revise(&mut self, previous: Option<&Self>, now: u64) -> Result<(), String> {
        self.upgrade()?;
        let mut comparable = self.clone();
        if let (Some(old), Some(care)) = (previous, comparable.care.as_mut()) {
            if let Some(old_care) = &old.care {
                care.exported_revision = old_care.exported_revision;
                care.encrypted_export = old_care.encrypted_export;
            }
            if comparable == *old {
                return Ok(());
            }
        }
        let care = self.care.as_mut().unwrap();
        care.revision = match previous.and_then(|p| p.care.as_ref()) {
            // TODO: localize
            Some(old) => old
                .revision
                .checked_add(1)
                .ok_or("Plan revision limit reached.")?,
            None => 1,
        };
        care.modified_at = if valid_date(Some(now)) {
            Some(now)
        } else {
            None
        };
        self.validate()?;
        Ok(())
    }

    pub fn export_status(&self) -> String {
        if self
            .care
            .as_ref()
            .is_some_and(|c| c.exported_revision.is_some() && !c.encrypted_export)
        {
            return "Only an older plaintext export is recorded. Create an encrypted backup; older copies remain readable.".into();
        }
        // TODO: localize
        match &self.care {
            Some(c) if c.exported_revision == Some(c.revision) => {
                format!(
                    "Revision {} backed up. Keep its password separate.",
                    c.revision
                )
            }
            Some(c) if c.exported_revision.is_some() => "Your backup copy is out of date.".into(),
            _ => "No encrypted backup recorded on this device.".into(),
        }
    }

    pub fn forget_imported_assurances(&mut self) {
        self.reset_review();
        if let Some(c) = &mut self.care {
            c.exported_revision = None;
            c.encrypted_export = false;
            c.rehearsed_at = None;
        }
    }

    pub fn record_rehearsal(&mut self, confirmed: &[bool], now: u64) -> Result<(), String> {
        if confirmed != [true, true, true] {
            // TODO: localize
            return Err("Confirm all three rehearsal steps before recording.".into());
        }
        let missing = self.completeness();
        if !missing.is_empty() {
            // TODO: localize
            return Err(format!(
                "Complete these plan details:\n{}",
                missing.join("\n")
            ));
        }
        if !valid_date(Some(now)) {
            // TODO: localize
            return Err("Check the device date before recording the rehearsal.".into());
        }
        self.upgrade()?;
        if self
            .care
            .as_ref()
            .unwrap()
            .rehearsed_at
            .is_some_and(|last| last > now)
        {
            // TODO: localize
            return Err("The clock moved back. Check the device date.".into());
        }
        self.care.as_mut().unwrap().rehearsed_at = Some(now);
        self.checklist.rehearsed = true;
        Ok(())
    }

    pub fn record_practice(&mut self, checks: [bool; 5], now: u64) -> Result<(), String> {
        self.upgrade()?;
        let care = self.care.as_ref().unwrap();
        if !valid_date(Some(now))
            || care.practice.as_ref().is_some_and(|p| p.at > now)
            || care.rehearsed_at.is_some_and(|t| t > now)
        {
            // TODO: localize
            return Err("Check the device date before saving practice.".into());
        }
        if checks.iter().all(|v| *v) && !self.completeness().is_empty() {
            // TODO: localize
            return Err(
                "Complete the missing plan details before recording all practice steps as done."
                    .into(),
            );
        }
        let complete = checks.iter().all(|v| *v);
        let care = self.care.as_mut().unwrap();
        care.practice = Some(Practice { at: now, checks });
        care.rehearsed_at = complete.then_some(now);
        self.checklist.rehearsed = complete;
        Ok(())
    }

    pub fn recovery_guide(&self) -> Result<String, String> {
        let guide = self.guide()?;
        Ok(guide.replace("[ ] Rehearse with the owner before relying on this kit.",
            "[ ] If anything is unclear, stop and contact the trusted person above. Do not guess or share secrets.\n\nOwner preparation (before handover)\nPractise with your heir while you can help. Make sure they know this plan exists, can obtain a backup and can obtain its password separately. Plan an alternative if a copy, device or contact is unavailable. These arrangements must exist outside this locked plan.")
            .replace("INHERITANCE RECOVERY KIT", "INHERITANCE RECOVERY GUIDE")
            + "\nPrivacy\nThis guide can reveal how to locate signing keys. Share only with intended recipients. Keep credentials separate. Extra copies help if one is lost but increase exposure. Decrypted files are plaintext. This guide coordinates recovery; it does not replace distributed custody.\n")
    }

    pub fn care_guide(&self, wallet: &Wallet) -> Result<String, String> {
        let c = self.care.as_ref().unwrap();
        let normal_count = wallet
            .inheritance
            .as_ref()
            .map_or(wallet.fingerprints.len(), |p| p.normal_count);
        // TODO: localize
        Ok(format!("INHERITANCE RECOVERY KIT\n\nPlan: {}\nWallet ID: {}\nRevision: {}\nRevision date (UTC): {}\nNetwork: {}\nHeir: {}\nTrusted contact: {}\n\nStart here\n{}\n\nSigning keys: {} of {} required\n{}\n\nAdditional instructions (optional)\n{}\n\nRecovery checklist\n[ ] Obtain the required signing devices and separate access instructions.\n[ ] Compare fingerprints with the wallet configuration.\n[ ] Open wallet.bsms from this decrypted archive in Nunchuk, or open the existing matching wallet. The guide is readable without the Inheritance app.\n[ ] Compare the network and first receive address below.\n[ ] Follow Nunchuk's recovery instructions; recovery still requires the signing keys.\n[ ] Rehearse with the owner before relying on this kit.\n\nFirst receive address:\n{}\n\nWallet descriptor:\n{}\n\nMissing plan details\n{}\n\nThis is an internal POC recovery guide, not a Nunchuk inheritance-service claim. The app cannot access the master seed, sign, unlock funds, verify physical key possession or guarantee recovery. Keep PINs, passwords and seed backups separate. Never enter seed words on a website or share them with support. A saved review or rehearsal is only a user confirmation.\n",
            self.wallet_name, self.wallet_id()?, c.revision, utc_date(c.modified_at), wallet.network, self.heir, self.contact, self.message,
            wallet.threshold, normal_count, self.signer_instructions(), self.access_guidance, wallet.first_address, wallet.descriptor,
            if self.completeness().is_empty() { "None detected. This is not proof of recoverability.".into() } else { self.completeness().join("\n") }))
    }
}

pub fn utc_date(timestamp: Option<u64>) -> String {
    if !valid_date(timestamp) {
        return "Check device date".into();
    }
    let Some(t) = timestamp else {
        return "Not recorded".into();
    };
    let mut days = t / 86400;
    let mut year = 1970u32;
    let leap = |y: u32| y.is_multiple_of(4) && (!y.is_multiple_of(100) || y.is_multiple_of(400));
    while days >= if leap(year) { 366 } else { 365 } {
        days -= if leap(year) { 366 } else { 365 };
        year += 1;
    }
    let mut month = 1;
    for length in [
        31,
        if leap(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ] {
        if days < length {
            break;
        }
        days -= length;
        month += 1;
    }
    format!("{year:04}-{month:02}-{:02}", days + 1)
}
