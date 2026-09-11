use std::{collections::BTreeSet, str::FromStr};

use miniscript::{
    bitcoin::{bip32::ChildNumber, secp256k1::Secp256k1, Network, NetworkKind},
    descriptor::{DescriptorPublicKey, Wildcard, WshInner},
    Descriptor, Terminal,
};
use serde::{Deserialize, Serialize};
mod care;
pub use care::*;
mod backup;
pub use backup::*;

pub const MAX_FILE: usize = 128 * 1024;
pub const MAX_DESCRIPTOR: usize = 8192;
pub const MAX_NOTE: usize = 1600;
const DAY: u64 = 86400;

// TODO: localize
const INVALID_WALLET: &str = "Choose a complete supported public multisig descriptor or BSMS file. This policy or export is not supported.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wallet {
    pub descriptor: String,
    pub threshold: usize,
    pub fingerprints: Vec<String>,
    pub network: String,
    pub first_address: String,
    pub inheritance: Option<Inheritance>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inheritance {
    pub threshold: usize,
    pub after: u32,
    pub normal_count: usize,
}

impl Wallet {
    pub fn summary(&self) -> String {
        if let Some(p) = &self.inheritance {
            format!(
                "Normal: {} of {}\nInheritance: {} of {} (timelocked)",
                self.threshold,
                p.normal_count,
                p.threshold,
                self.fingerprints.len() - p.normal_count
            )
        } else {
            self.signing_rules()
        }
    }

    pub fn signer_role(&self, index: usize) -> &'static str {
        match &self.inheritance {
            Some(p) if index >= p.normal_count => "Inheritance key",
            Some(_) => "Normal key",
            None => "Signing key",
        }
    }

    pub fn signing_rules(&self) -> String {
        match &self.inheritance {
            None => format!("{} of {} signing keys required", self.threshold, self.fingerprints.len()),
            Some(p) => format!("Normal: {} of {} keys\nInheritance: {} of {} inheritance keys\n{}\nThe normal path remains available. Compatible wallet software must check blockchain eligibility; this app does not unlock funds.", self.threshold, p.normal_count, p.threshold, self.fingerprints.len() - p.normal_count,
                if p.after < 500_000_000 { format!("After block height {}", p.after) } else { format!("After Unix time {} ({} UTC date). Eligibility uses blockchain median time, not this device's clock.", p.after, utc_date(Some(p.after as u64))) }),
        }
    }
}

pub fn parse_wallet(input: &str) -> Result<Wallet, String> {
    if input.len() > MAX_DESCRIPTOR || !input.is_ascii() {
        return Err(INVALID_WALLET.into());
    }
    let lines: Vec<_> = input.trim().lines().map(str::trim).collect();
    let (raw, expected_address) = if lines.first() == Some(&"BSMS 1.0") {
        if lines.len() != 4 || !["/0/*,/1/*", "No path restrictions"].contains(&lines[2]) {
            // TODO: localize
            return Err(
                "Expected a four-line BSMS 1.0 wallet export with receive and change paths.".into(),
            );
        }
        (lines[1], Some(lines[3]))
    } else if lines.len() == 1 {
        (lines[0], None)
    } else {
        return Err(INVALID_WALLET.into());
    };
    if !raw.starts_with("wsh(") {
        return Err(INVALID_WALLET.into());
    }
    let body = if let Some((body, checksum)) = raw.split_once('#') {
        if miniscript::descriptor::checksum::desc_checksum(body)
            .map_err(|_| INVALID_WALLET.to_string())?
            != checksum
        {
            return Err(INVALID_WALLET.into());
        }
        body
    } else if expected_address.is_some() && raw.contains("/**") {
        raw
    } else {
        return Err(INVALID_WALLET.into());
    };
    if expected_address.is_some() && lines[2] == "No path restrictions" && body.contains("/**") {
        return Err(INVALID_WALLET.into());
    }
    // BSMS templates use the separately validated receive/change restrictions.
    let expanded = if expected_address.is_some() {
        body.replace("/**", "/<0;1>/*")
    } else {
        body.to_string()
    };
    let descriptor = Descriptor::<DescriptorPublicKey>::from_str(&expanded)
        .map_err(|_| INVALID_WALLET.to_string())?;
    descriptor
        .sanity_check()
        .map_err(|_| INVALID_WALLET.to_string())?;
    let Descriptor::Wsh(wsh) = &descriptor else {
        return Err(INVALID_WALLET.into());
    };
    let (threshold, keys, inheritance) = match wsh.as_inner() {
        WshInner::SortedMulti(multi) if multi.k() >= 2 && multi.n() <= 5 => {
            (multi.k(), multi.pks().to_vec(), None)
        }
        WshInner::Ms(ms) => {
            let Terminal::OrD(normal, recovery) = &ms.node else {
                return Err(INVALID_WALLET.into());
            };
            let Terminal::Multi(normal) = &normal.node else {
                return Err(INVALID_WALLET.into());
            };
            let Terminal::AndV(verified, lock) = &recovery.node else {
                return Err(INVALID_WALLET.into());
            };
            let Terminal::Verify(recovery) = &verified.node else {
                return Err(INVALID_WALLET.into());
            };
            let Terminal::Multi(recovery) = &recovery.node else {
                return Err(INVALID_WALLET.into());
            };
            let Terminal::After(lock) = &lock.node else {
                return Err(INVALID_WALLET.into());
            };
            if normal.k() < 2 || normal.n() > 5 || recovery.n() > 5 {
                return Err(INVALID_WALLET.into());
            }
            let mut keys = normal.data().to_vec();
            keys.extend_from_slice(recovery.data());
            (
                normal.k(),
                keys,
                Some(Inheritance {
                    threshold: recovery.k(),
                    after: lock.to_consensus_u32(),
                    normal_count: normal.n(),
                }),
            )
        }
        _ => return Err(INVALID_WALLET.into()),
    };
    let mut fingerprints = Vec::new();
    let mut seen = BTreeSet::new();
    let mut seen_fingerprints = BTreeSet::new();
    let mut network = None;
    for key in &keys {
        let (xpub, origin, wildcard, paths) = match key {
            DescriptorPublicKey::XPub(k) => (
                k.xkey,
                &k.origin,
                k.wildcard,
                vec![k.derivation_path.clone()],
            ),
            DescriptorPublicKey::MultiXPub(k) => (
                k.xkey,
                &k.origin,
                k.wildcard,
                k.derivation_paths.paths().clone(),
            ),
            _ => return Err(INVALID_WALLET.into()),
        };
        let Some((fingerprint, _)) = origin else {
            // TODO: localize
            return Err(
                "Every signing key needs its origin fingerprint in the wallet export.".into(),
            );
        };
        let paths: Vec<Vec<_>> = paths
            .iter()
            .map(|p| p.into_iter().copied().collect())
            .collect();
        let receive = vec![ChildNumber::Normal { index: 0 }];
        let change = vec![ChildNumber::Normal { index: 1 }];
        if wildcard != Wildcard::Unhardened || paths != [receive, change] {
            // TODO: localize
            return Err(
                "Export a complete receive-and-change (/<0;1>/*) descriptor or BSMS wallet file. A receive-only export cannot recover change."
                    .into(),
            );
        }
        let signer_id = if inheritance.is_some() {
            format!(
                "{} / {}",
                fingerprint.to_string().to_uppercase(),
                origin.as_ref().unwrap().1
            )
        } else {
            fingerprint.to_string().to_uppercase()
        };
        if !seen.insert(xpub.public_key) || !seen_fingerprints.insert(signer_id.clone()) {
            // TODO: localize
            return Err(
                "Each signer must have a different public key and origin fingerprint.".into(),
            );
        }
        if network.is_some_and(|n| n != xpub.network) {
            // TODO: localize
            return Err("The wallet mixes mainnet and test-network keys.".into());
        }
        network = Some(xpub.network);
        fingerprints.push(signer_id);
    }
    let net = if network == Some(NetworkKind::Main) {
        Network::Bitcoin
    } else {
        Network::Testnet
    };
    let first = descriptor
        .clone()
        .into_single_descriptors()
        .map_err(|_| INVALID_WALLET.to_string())?
        .remove(0)
        .at_derivation_index(0)
        .map_err(|_| INVALID_WALLET.to_string())?
        .derived_descriptor(&Secp256k1::verification_only())
        .map_err(|_| INVALID_WALLET.to_string())?
        .address(net)
        .map_err(|_| INVALID_WALLET.to_string())?
        .to_string();
    if expected_address.is_some_and(|address| address != first) {
        // TODO: localize
        return Err(
            "The BSMS first address does not match its descriptor. Export the wallet again.".into(),
        );
    }
    Ok(Wallet {
        descriptor: descriptor.to_string(),
        threshold,
        fingerprints,
        // TODO: localize
        network: if net == Network::Bitcoin {
            "Bitcoin mainnet"
        } else {
            "Test network (not real bitcoin)"
        }
        .into(),
        first_address: first,
        inheritance,
    })
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checklist {
    pub keys_located: bool,
    pub access_arranged: bool,
    pub backup_shared: bool,
    pub rehearsed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub version: u32,
    pub wallet_name: String,
    pub descriptor: String,
    pub heir: String,
    pub contact: String,
    pub message: String,
    pub key_guidance: String,
    pub access_guidance: String,
    pub review_days: u32,
    pub last_reviewed: Option<u64>,
    pub checklist: Checklist,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub care: Option<Care>,
}

impl Plan {
    pub fn new(wallet: &Wallet) -> Self {
        Self {
            version: 1,
            wallet_name: String::new(),
            descriptor: wallet.descriptor.clone(),
            heir: String::new(),
            contact: String::new(),
            message: String::new(),
            key_guidance: String::new(),
            access_guidance: String::new(),
            review_days: 90,
            last_reviewed: None,
            checklist: Checklist::default(),
            care: None,
        }
    }

    pub fn validate(&self) -> Result<Wallet, String> {
        if self.version != 1 {
            // TODO: localize
            return Err(
                "This plan version is not supported. Update the app before importing it.".into(),
            );
        }
        for (name, value, max) in [
            ("Wallet name", &self.wallet_name, 80),
            ("Heir name", &self.heir, 80),
            ("Contact", &self.contact, 200),
            ("Message", &self.message, MAX_NOTE),
            ("Key guidance", &self.key_guidance, MAX_NOTE),
            ("Access guidance", &self.access_guidance, MAX_NOTE),
        ] {
            if value.chars().count() > max
                || value.chars().any(|c| {
                    (c.is_control() && c != '\n')
                        || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
                })
            {
                // TODO: localize
                return Err(format!("{name} must be at most {max} characters and contain no hidden control characters."));
            }
        }
        if ![30, 90, 180, 365].contains(&self.review_days) {
            // TODO: localize
            return Err("Choose a review interval of 30, 90, 180 or 365 days.".into());
        }
        if self
            .last_reviewed
            .is_some_and(|t| !(1577836800..=4102444800).contains(&t))
        {
            // TODO: localize
            return Err("The review date is outside the supported range (2020 to 2100).".into());
        }
        let wallet = parse_wallet(&self.descriptor)?;
        if let Some(care) = &self.care {
            care.validate(&wallet)?;
        }
        Ok(wallet)
    }

    pub fn has_guidance(&self) -> bool {
        if self.care.is_some() {
            return self.completeness().is_empty();
        }
        [&self.wallet_name, &self.heir, &self.key_guidance]
            .iter()
            .all(|s| !s.trim().is_empty())
    }

    pub fn reset_review(&mut self) {
        self.last_reviewed = None;
        self.checklist = Checklist::default();
        if let Some(c) = &mut self.care {
            c.rehearsed_at = None;
            c.practice = None;
            c.handover = [false; 4];
        }
    }

    pub fn review_status(&self, now: u64) -> String {
        // TODO: localize
        if !(1577836800..=4102444800).contains(&now) {
            return "Check the device date".into();
        }
        let Some(last) = self.last_reviewed else {
            // TODO: localize
            return "Not reviewed yet".into();
        };
        // TODO: localize
        if now < last {
            return "Clock moved back: check device date".into();
        }
        let due = last + u64::from(self.review_days) * DAY;
        // TODO: localize
        if now >= due {
            return "Plan review due".into();
        }
        // TODO: localize
        let days = (due - now).div_ceil(DAY);
        format!(
            "Review in {days} {}",
            if days == 1 { "day" } else { "days" }
        )
    }

    pub fn mark_reviewed(&mut self, now: u64) -> Result<(), String> {
        self.validate()?;
        if !self.has_guidance()
            || !self.checklist.keys_located
            || !self.checklist.access_arranged
            || !self.checklist.backup_shared
        {
            // TODO: localize
            return Err(
                "Complete the plan and the first three checks before recording a review.".into(),
            );
        }
        if !(1577836800..=4102444800).contains(&now)
            || self.last_reviewed.is_some_and(|last| last > now)
        {
            // TODO: localize
            return Err("Check the device date before recording a review.".into());
        }
        self.last_reviewed = Some(now);
        Ok(())
    }

    pub fn bsms(&self) -> Result<String, String> {
        let wallet = self.validate()?;
        if wallet.inheritance.is_some() {
            return Ok(format!(
                "BSMS 1.0\n{}\nNo path restrictions\n{}\n",
                wallet.descriptor, wallet.first_address
            ));
        }
        let template = wallet
            .descriptor
            .split('#')
            .next()
            .unwrap()
            .replace("/<0;1>/*", "/**");
        Ok(format!(
            "BSMS 1.0\n{template}\n/0/*,/1/*\n{}\n",
            wallet.first_address
        ))
    }

    pub fn guide(&self) -> Result<String, String> {
        let wallet = self.validate()?;
        if wallet.inheritance.is_some() && self.care.is_none() {
            let mut upgraded = self.clone();
            upgraded.upgrade()?;
            return upgraded.guide();
        }
        if self.care.is_some() {
            return self.care_guide(&wallet);
        }
        // TODO: localize
        Ok(format!("INHERITANCE RECOVERY GUIDE\n\nWallet: {}\nNetwork: {}\nFor: {}\nContact: {}\n\n1. Find {} of the {} signing keys.\nFingerprints: {}\n{}\n\n2. Additional instructions (optional).\n{}\n\n3. Import the wallet configuration below into compatible wallet software, or open the existing matching wallet. Compare its policy, network and first receive address before continuing.\n\n4. Connect enough supported hardware signers to meet the threshold. Follow the recovery instructions for your wallet software.\n\n5. Rehearse with the owner first. Never type seed words into a website or share them with support.\n\nMessage:\n{}\n\nFirst receive address:\n{}\n\nWallet descriptor:\n{}\n\nThis kit contains public wallet data and personal guidance. It contains no automatic release mechanism and is not an inheritance-service claim. Required keys must be obtained separately. A review date never changes the wallet's signing rules.\n",
            self.wallet_name, wallet.network, self.heir, self.contact, wallet.threshold,
            wallet.fingerprints.len(), wallet.fingerprints.join(", "), self.key_guidance,
            self.access_guidance, self.message, wallet.first_address, wallet.descriptor))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Kit {
    format: String,
    plan: Plan,
    guide: String,
}

pub fn export_kit(plan: &Plan) -> Result<Vec<u8>, String> {
    let kit = Kit {
        format: if plan.care.is_some() {
            "passport-inheritance-kit-v2"
        } else {
            "passport-inheritance-kit-v1"
        }
        .into(),
        plan: plan.clone(),
        guide: plan.guide()?,
    };
    let bytes = serde_json::to_vec_pretty(&kit).map_err(|e| e.to_string())?;
    if bytes.len() > MAX_FILE {
        // TODO: localize
        return Err("The kit is too large. Shorten the recovery notes.".into());
    }
    Ok(bytes)
}

pub fn import_kit(bytes: &[u8]) -> Result<Plan, String> {
    if bytes.len() > MAX_FILE {
        // TODO: localize
        return Err("The recovery kit exceeds 128 KiB.".into());
    }
    let kit: Kit = serde_json::from_slice(bytes).map_err(|_| {
        // TODO: localize
        "Choose a recovery kit exported by this app.".to_string()
    })?;
    if kit.format
        != if kit.plan.care.is_some() {
            "passport-inheritance-kit-v2"
        } else {
            "passport-inheritance-kit-v1"
        }
    {
        // TODO: localize
        return Err("Unsupported recovery kit format.".into());
    }
    let generated = kit.plan.guide()?;
    if kit.guide != generated {
        // TODO: localize
        return Err("The kit's guide differs from its plan. Review the original export.".into());
    }
    Ok(kit.plan)
}

pub fn encode_saved(plan: &Option<Plan>) -> Result<Vec<u8>, String> {
    if let Some(plan) = plan {
        plan.validate()?;
    }
    serde_json::to_vec(plan).map_err(|e| e.to_string())
}

pub fn decode_saved(bytes: &[u8]) -> Result<Option<Plan>, String> {
    if bytes.len() > MAX_FILE {
        return Err("Saved plan is too large.".into());
    }
    let plan: Option<Plan> =
        serde_json::from_slice(bytes).map_err(|_| "Saved plan could not be read.".to_string())?;
    if let Some(plan) = &plan {
        plan.validate()?;
    }
    Ok(plan)
}

pub const MAX_PLANS: usize = 20;
const MAX_LIBRARY: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub id: u64,
    pub plan: Plan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Library {
    version: u32,
    next_id: u64,
    pub entries: Vec<Entry>,
}

impl Default for Library {
    fn default() -> Self {
        Self {
            version: 2,
            next_id: 1,
            entries: Vec::new(),
        }
    }
}

impl Library {
    pub fn validate(&self) -> Result<(), String> {
        let mut ids = BTreeSet::new();
        if self.version != 2 || self.entries.len() > MAX_PLANS || self.next_id == 0 {
            // TODO: localize
            return Err("Unsupported or invalid plan collection.".into());
        }
        for entry in &self.entries {
            if entry.id == 0 || entry.id >= self.next_id || !ids.insert(entry.id) {
                // TODO: localize
                return Err("Invalid plan identity in saved collection.".into());
            }
            entry.plan.validate()?;
        }
        Ok(())
    }

    pub fn put(&mut self, id: Option<u64>, plan: Plan) -> Result<u64, String> {
        plan.validate()?;
        if let Some(id) = id {
            let entry = self
                .entries
                .iter_mut()
                .find(|e| e.id == id)
                // TODO: localize
                .ok_or("The selected plan no longer exists.")?;
            entry.plan = plan;
            Ok(id)
        } else {
            if self.entries.len() >= MAX_PLANS {
                // TODO: localize
                return Err(
                    "You can save up to 20 plans. Export and delete a plan to make room.".into(),
                );
            }
            let id = self.next_id;
            // TODO: localize
            let next = id
                .checked_add(1)
                .ok_or("No more plan identities available.")?;
            self.entries.push(Entry { id, plan });
            self.next_id = next;
            Ok(id)
        }
    }

    pub fn remove(&mut self, id: u64) -> Result<(), String> {
        let index = self
            .entries
            .iter()
            .position(|e| e.id == id)
            // TODO: localize
            .ok_or("The selected plan no longer exists.")?;
        self.entries.remove(index);
        Ok(())
    }
}

pub fn encode_library(library: &Library) -> Result<Vec<u8>, String> {
    library.validate()?;
    let bytes = serde_json::to_vec(library).map_err(|e| e.to_string())?;
    if bytes.len() > MAX_LIBRARY {
        // TODO: localize
        return Err("Saved plans exceed the storage limit.".into());
    }
    Ok(bytes)
}

pub fn decode_library(bytes: &[u8]) -> Result<Library, String> {
    if bytes.len() > MAX_LIBRARY {
        // TODO: localize
        return Err("Saved plans exceed the storage limit.".into());
    }
    let value: serde_json::Value = serde_json::from_slice(bytes)
        // TODO: localize
        .map_err(|_| "Saved plans could not be read.")?;
    if value.is_null() || value.get("version").and_then(|v| v.as_u64()) == Some(1) {
        let mut library = Library::default();
        if let Some(plan) = decode_saved(bytes)? {
            library.put(None, plan)?;
        }
        return Ok(library);
    }
    let library: Library = serde_json::from_value(value)
        // TODO: localize
        .map_err(|_| "Saved plans could not be read.")?;
    library.validate()?;
    Ok(library)
}
