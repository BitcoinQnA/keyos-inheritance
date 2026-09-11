use crate::{gui_permissions::GuiPermissions, Actions, AppWindow, View};
use inheritance_core::{
    decode_library, decrypt_backup, encode_library, encrypt_backup, parse_wallet,
    validate_backup_password, validate_encrypted_backup, Library, Plan, Wallet, MAX_BACKUP,
};
use slint_keyos_platform::{
    gui_server_api::navigation::{
        filepicker::{AllowedExtensions, AllowedLocations, Location, SelectFileOptions},
        qrscanner::{ScanQrOptions, ScanQrResult},
    },
    navigation::{open_qr_scanner, select_file},
    slint::{ComponentHandle, ModelRc, VecModel},
};
use std::{
    cell::RefCell,
    io::{Read, Write},
    rc::Rc,
    sync::{mpsc, Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use zeroize::Zeroizing;
fs::use_api!();
const PLAN_FILE: &str = "inheritance-plan.json";

#[derive(Default)]
struct State {
    library: Library,
    selected: Option<u64>,
    saved: Option<Plan>,
    draft: Option<Plan>,
    pending: Option<Plan>,
    load_error: bool,
    edit_return: i32,
    rehearsal: [bool; 5],
    handover: [bool; 4],
    review: [bool; 3],
    incoming: Option<Vec<u8>>,
    crypto_result: Arc<Mutex<Option<Result<CryptoResult, String>>>>,
    crypto_sender: Option<mpsc::Sender<CryptoWork>>,
}
type CryptoWork = Box<dyn FnOnce() -> Result<CryptoResult, String> + Send>;

enum CryptoResult {
    Encrypted { plan: Plan, bytes: Vec<u8> },
    Decrypted(Plan),
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
fn fail(ui: &AppWindow, error: impl ToString) {
    ui.global::<View>().set_error(error.to_string().into());
}
fn page(ui: &AppWindow, number: i32) {
    let v = ui.global::<View>();
    v.set_error("".into());
    v.set_notice("".into());
    v.set_page(number);
    v.set_backup_password("".into());
    v.set_backup_confirmation("".into());
    // TODO: localize
    v.set_title(
        match number {
            1 => "Import Wallet",
            2 => "Wallet Details",
            3 => "Recovery Plan",
            4 => "Edit Details",
            5 => "Review Details",
            6 => "Heir Guidance",
            7 => "Encrypted Backup",
            8 => "Restore Plan",
            9 => "About",
            10 => "Delete Plan",
            11 => "Plan Details",
            12 => "Set Up Your Plan",
            13 => "Your Signing Keys",
            14 => "Signing Key",
            15 => "Heir Guidance",
            16 => "Practise Together",
            17 => "Check Your Plan",
            18 => "Unlock Backup",
            19 => "Handover Arrangements",
            _ => "Plans",
        }
        .into(),
    );
}
fn persist(library: &Library) -> Result<(), String> {
    FileSystem::default()
        .durable_file_write(PLAN_FILE, fs::Location::AppData, &encode_library(library)?)
        // TODO: localize
        .map_err(|_| "Could not save the plan. Please try again.".into())
}
fn wallet_view(ui: &AppWindow, wallet: &Wallet) {
    let v = ui.global::<View>();
    // TODO: localize
    v.set_wallet_summary(format!("{}\n{}", wallet.summary(), wallet.network).into());
    v.set_fingerprints(wallet.fingerprints.join("\n").into());
    v.set_address(wallet.first_address.clone().into());
}
fn sync(ui: &AppWindow, plan: &Plan) {
    let v = ui.global::<View>();
    let mut editable = plan.clone();
    let _ = editable.upgrade();
    let missing = editable.completeness();
    v.set_complete(missing.is_empty());
    // TODO: localize
    v.set_completeness(if missing.is_empty() { "Required fields are filled in. This does not prove the instructions work, keys are available or recovery will succeed.".into() } else { missing.join("\n\n").into() });
    v.set_export_status(plan.export_status().into());
    v.set_export_current(
        plan.care
            .as_ref()
            .is_some_and(|c| c.encrypted_export && c.exported_revision == Some(c.revision)),
    );
    if let Some(c) = &editable.care {
        // TODO: localize
        v.set_handover_status(
            format!(
                "{} of 4 arrangements confirmed",
                c.handover.iter().filter(|v| **v).count()
            )
            .into(),
        );
        // TODO: localize
        v.set_revision(
            format!(
                "Revision {} · {} UTC",
                c.revision,
                inheritance_core::utc_date(c.modified_at)
            )
            .into(),
        );
        v.set_rehearsal_status(
            c.rehearsed_at
                .map(|t| {
                    format!(
                        "Last practised: {} UTC (user confirmed)",
                        inheritance_core::utc_date(Some(t))
                    )
                })
                .unwrap_or_else(|| "Not practised yet".into())
                .into(),
        );
        if let Some(practice) = &c.practice {
            // TODO: localize
            v.set_rehearsal_status(
                format!(
                    "{} of 5 steps done · {} UTC\nUser confirmations, not proof of recovery",
                    practice.checks.iter().filter(|v| **v).count(),
                    inheritance_core::utc_date(Some(practice.at))
                )
                .into(),
            );
        } else if c.rehearsed_at.is_some() {
            // TODO: localize
            v.set_rehearsal_status(
                "Older practice recorded. Run the expanded five-step check.".into(),
            );
        }
        v.set_signer_names(ModelRc::new(VecModel::from(
            c.signers
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    if s.name.is_empty() {
                        format!(
                            "{} {}",
                            plan.validate().map_or("Signing key", |w| w.signer_role(i)),
                            i + 1
                        )
                        .into()
                    } else {
                        s.name.clone().into()
                    }
                })
                .collect::<Vec<_>>(),
        )));
        v.set_signer_summaries(ModelRc::new(VecModel::from(
            c.signers
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    format!(
                        "{}\n{}\n{}",
                        plan.validate().map_or("Signing key", |w| w.signer_role(i)),
                        s.fingerprint,
                        if s.location.is_empty() || s.access.is_empty() {
                            "Instructions needed"
                        } else {
                            "Location and access recorded"
                        }
                    )
                    .into()
                })
                .collect::<Vec<_>>(),
        )));
    }
    if let Ok(wallet) = plan.validate() {
        wallet_view(ui, &wallet);
    }
    // TODO: localize
    v.set_wallet_name(if plan.wallet_name.is_empty() {
        "Unnamed wallet".into()
    } else {
        plan.wallet_name.clone().into()
    });
    // TODO: localize
    v.set_heir(if plan.heir.is_empty() {
        "Add your heir and instructions".into()
    } else {
        format!("For {}", plan.heir).into()
    });
    // TODO: localize
    v.set_status(match plan.last_reviewed {
        Some(date) => format!(
            "Last reviewed: {} UTC\n{}",
            inheritance_core::utc_date(Some(date)),
            plan.review_status(now())
        )
        .into(),
        None => plan.review_status(now()).into(),
    });
    // TODO: localize
    v.set_readiness(
        if !plan.has_guidance() {
            "Details still needed"
        } else if !plan.checklist.rehearsed {
            "Practice with your heir is optional"
        } else {
            "Review your recovery checklist"
        }
        .into(),
    );
    let fields: Vec<_> = [
        &plan.wallet_name,
        &plan.heir,
        &plan.contact,
        &plan.message,
        &plan.key_guidance,
        &plan.access_guidance,
    ]
    .iter()
    .map(|s| s.as_str().into())
    .collect();
    v.set_fields(ModelRc::new(VecModel::from(fields)));
    v.set_checks(ModelRc::new(VecModel::from(vec![
        plan.checklist.keys_located,
        plan.checklist.access_arranged,
        plan.checklist.backup_shared,
    ])));
    v.set_review_days(plan.review_days as i32);
}
fn home(ui: &AppWindow, state: &mut State) {
    page(ui, 0);
    state.draft = None;
    state.pending = None;
    state.selected = None;
    state.saved = None;
    let v = ui.global::<View>();
    v.set_has_plan(!state.library.entries.is_empty());
    v.set_plan_names(ModelRc::new(VecModel::from(
        state
            .library
            .entries
            .iter()
            .map(|e| {
                // TODO: localize
                if e.plan.wallet_name.is_empty() {
                    "Unnamed plan".into()
                } else {
                    e.plan.wallet_name.clone().into()
                }
            })
            .collect::<Vec<_>>(),
    )));
    v.set_plan_summaries(ModelRc::new(VecModel::from(
        state
            .library
            .entries
            .iter()
            .map(|e| {
                // TODO: localize
                format!(
                    "{}\n{}\n{}",
                    if e.plan.heir.is_empty() {
                        "Add an heir".into()
                    } else {
                        format!("For {}", e.plan.heir)
                    },
                    e.plan.review_status(now()),
                    e.plan
                        .validate()
                        .map(|w| if w.network.starts_with("Bitcoin") {
                            "Mainnet"
                        } else {
                            "Test network"
                        })
                        .unwrap_or("Invalid wallet")
                )
                .into()
            })
            .collect::<Vec<_>>(),
    )));
    if state.load_error {
        // TODO: localize
        fail(
            ui,
            "The saved plans could not be read. No changes will be written. Keep your encrypted backups and contact support.",
        );
    }
}
fn detail(ui: &AppWindow, state: &mut State) {
    state.draft = None;
    state.pending = None;
    if let Some(plan) = &state.saved {
        page(ui, 11);
        sync(ui, plan);
    } else {
        home(ui, state);
    }
}
fn commit(ui: &AppWindow, state: &mut State, mut plan: Option<Plan>) -> bool {
    if state.load_error {
        // TODO: localize
        fail(ui, "Saved plans are unreadable. No changes were made.");
        return false;
    }
    let mut library = state.library.clone();
    if let Some(p) = &mut plan {
        if let Err(e) = p.revise(state.selected.and(state.saved.as_ref()), now()) {
            fail(ui, e);
            return false;
        }
    }
    let result = match &plan {
        Some(plan) => library.put(state.selected, plan.clone()).map(Some),
        None => state
            .selected
            .map(|id| library.remove(id))
            .transpose()
            .map(|_| None),
    };
    let selected = match result {
        Ok(id) => id,
        Err(e) => {
            fail(ui, e);
            return false;
        }
    };
    if let Err(error) = persist(&library) {
        fail(ui, error);
        return false;
    }
    state.saved = plan;
    state.library = library;
    state.selected = selected;
    state.load_error = false;
    true
}

pub fn init(ui: &AppWindow) {
    let mut initial = State::default();
    match FileSystem::default().durable_file_read(PLAN_FILE, fs::Location::AppData) {
        Ok(bytes) => match decode_library(&bytes) {
            Ok(library) => initial.library = library,
            Err(_) => initial.load_error = true,
        },
        Err(fs::Error::FileNotFound) => {}
        Err(_) => initial.load_error = true,
    }
    home(ui, &mut initial);
    let state = Rc::new(RefCell::new(initial));
    let actions = ui.global::<Actions>();
    macro_rules! on {
        ($method:ident, |$ui:ident, $state:ident $(, $arg:ident)*| $body:block) => {
            actions.$method({
                let weak = ui.as_weak(); let shared = state.clone();
                move |$($arg),*| {
                    let Some($ui) = weak.upgrade() else { return };
                    if $ui.global::<View>().get_crypto_busy() && stringify!($method) != "on_crypto_finished" { return; }
                    let mut borrow = shared.borrow_mut();
                    let $state = &mut *borrow;
                    $ui.global::<View>().set_error("".into());
                    $body
                }
            });
        }
    }
    on!(on_select_plan, |ui, state, index| {
        if let Some(entry) = state.library.entries.get(index as usize) {
            state.selected = Some(entry.id);
            state.saved = Some(entry.plan.clone());
            detail(&ui, state);
        }
    });
    on!(on_go, |ui, state, destination| {
        if destination == 0 {
            home(&ui, state);
            return;
        }
        if destination == 11 {
            detail(&ui, state);
            return;
        }
        if destination == 1 {
            if state.library.entries.len() >= inheritance_core::MAX_PLANS {
                // TODO: localize
                fail(
                    &ui,
                    "You can save up to 20 plans. Export and delete one to make room.",
                );
                return;
            }
            state.selected = None;
            state.saved = None;
            state.draft = None;
        }
        if destination == 3 || destination == 12 {
            state.draft = state.saved.clone();
            if let Some(p) = &mut state.draft {
                let _ = p.upgrade();
            }
            ui.global::<View>().set_setup_step(0);
        }
        if destination == 13 && state.draft.is_none() {
            state.draft = state.saved.clone();
            if let Some(p) = &mut state.draft {
                let _ = p.upgrade();
            }
        }
        page(&ui, destination);
        if let Some(plan) = &state.saved {
            sync(&ui, plan);
        }
        if destination == 12 || destination == 13 {
            if let Some(p) = &state.draft {
                sync(&ui, p);
            }
        }
        if destination == 6 {
            guide(&ui, state, 0);
        }
        if (destination == 5 || destination == 16)
            && state
                .saved
                .as_ref()
                .is_some_and(|p| !p.completeness().is_empty())
        {
            page(&ui, 17);
            // TODO: localize
            ui.global::<View>()
                .set_notice("Complete the missing details below, then check your plan.".into());
            return;
        }
        if destination == 5 {
            state.review = [false; 3];
            ui.global::<View>()
                .set_checks(ModelRc::new(VecModel::from(state.review.to_vec())));
        }
        if destination == 16 {
            state.rehearsal = [false; 5];
            rehearsal_view(&ui, state, 0);
        }
        if destination == 19 {
            state.handover = state
                .saved
                .as_ref()
                .and_then(|p| p.care.as_ref())
                .map_or([false; 4], |c| c.handover);
            ui.global::<View>()
                .set_handover_checks(ModelRc::new(VecModel::from(state.handover.to_vec())));
        }
    });
    on!(on_back, |ui, state| {
        let v = ui.global::<View>();
        if v.get_page() == 18 {
            state.incoming = None;
        }
        match v.get_page() {
            4 => {
                page(&ui, state.edit_return);
                if let Some(p) = &state.draft {
                    sync(&ui, p);
                    signer_view(&ui, p);
                }
            }
            14 => page(&ui, 13),
            13 => page(&ui, 12),
            12 if v.get_setup_step() > 0 => {
                v.set_setup_step(v.get_setup_step() - 1);
            }
            16 if v.get_rehearsal_step() > 0 => {
                rehearsal_view(&ui, state, v.get_rehearsal_step() - 1)
            }
            6 if v.get_guide_step() > 0 => guide(&ui, state, v.get_guide_step() - 1),
            6 => page(&ui, 15),
            5 | 16 | 19 => page(&ui, 17),
            2 => {
                state.pending = None;
                page(&ui, 1);
            }
            0 | 1 | 8 | 9 | 11 | 18 => home(&ui, state),
            _ => detail(&ui, state),
        }
    });
    on!(on_import_wallet, |ui, state, qr| {
        if state.load_error {
            // TODO: localize
            fail(&ui, "Saved plans are unreadable. No changes were made.");
            return;
        }
        let result = if qr {
            scan()
        } else {
            pick(false).and_then(|p| p.map(|p| read(&p)).transpose())
        };
        match result.and_then(|bytes| {
            bytes
                .map(|b| {
                    std::str::from_utf8(&b)
                        .map_err(|_| "Choose a text wallet export.".to_string())
                        .and_then(parse_wallet)
                })
                .transpose()
        }) {
            Ok(Some(wallet)) => {
                state.pending = Some(Plan::new(&wallet));
                page(&ui, 2);
                wallet_view(&ui, &wallet);
                duplicate_view(&ui, state);
            }
            Ok(None) => {}
            Err(error) => fail(&ui, error),
        }
    });
    on!(on_accept_wallet, |ui, state| {
        if let Some(mut plan) = state.pending.take() {
            if let Err(e) = plan.upgrade() {
                fail(&ui, e);
                return;
            }
            sync(&ui, &plan);
            state.draft = Some(plan);
            ui.global::<View>().set_setup_step(0);
            page(&ui, 12);
        }
    });
    on!(on_edit, |ui, state, index| {
        if !(0..6).contains(&index) {
            return;
        }
        let Some(plan) = &state.draft else { return };
        state.edit_return = ui.global::<View>().get_page();
        let values = [
            &plan.wallet_name,
            &plan.heir,
            &plan.contact,
            &plan.message,
            &plan.key_guidance,
            &plan.access_guidance,
        ];
        page(&ui, 4);
        let v = ui.global::<View>();
        v.set_field_index(index);
        v.set_field_multiline(index >= 3);
        v.set_edit_value(values[index as usize].as_str().into());
        // TODO: localize
        v.set_title(
            [
                "Plan Name",
                "Heir Name",
                "Trusted Contact",
                "Personal Message",
                "Additional Key Notes",
                "Additional Instructions",
            ][index as usize]
                .into(),
        );
        // TODO: localize
        v.set_field_help([
            "Use a name your family will recognise. Up to 80 characters.",
            "Who should follow this recovery plan? Up to 80 characters.",
            "A name, email or phone number for someone who can help. Up to 200 characters.",
            "A personal note for your heir. Do not include secrets. Up to 1600 characters.",
            "Optional notes applying to several keys. Record individual locations and access under Your Signing Keys. Never include seed words. Up to 1600 characters.",
            "Optional extra guidance not covered by the individual signing key instructions. Leave blank if unnecessary. Never include PINs, passwords or seed words. Up to 1600 characters.",
        ][index as usize].into());
    });
    on!(on_save_field, |ui, state| {
        let Some(mut plan) = state.draft.clone() else {
            return;
        };
        let v = ui.global::<View>();
        let index = v.get_field_index();
        if index >= 100 {
            let signer_index = ((index - 100) / 3) as usize;
            let field_index = (index - 100) % 3;
            let Some(signer) = plan
                .care
                .as_mut()
                .and_then(|c| c.signers.get_mut(signer_index))
            else {
                return;
            };
            let field = match field_index {
                0 => &mut signer.name,
                1 => &mut signer.location,
                _ => &mut signer.access,
            };
            *field = v.get_edit_value().trim().to_string();
            if let Err(e) = plan.validate() {
                fail(&ui, e);
                return;
            }
            state.draft = Some(plan.clone());
            page(&ui, 14);
            sync(&ui, &plan);
            signer_view(&ui, &plan);
            return;
        }
        let field = match index {
            0 => &mut plan.wallet_name,
            1 => &mut plan.heir,
            2 => &mut plan.contact,
            3 => &mut plan.message,
            4 => &mut plan.key_guidance,
            5 => &mut plan.access_guidance,
            _ => return,
        };
        *field = v.get_edit_value().trim().to_string();
        if let Err(e) = plan.validate() {
            fail(&ui, e);
            return;
        }
        state.draft = Some(plan.clone());
        page(&ui, state.edit_return);
        sync(&ui, &plan);
    });
    on!(on_interval, |ui, state| {
        if let Some(plan) = &mut state.draft {
            plan.review_days = match plan.review_days {
                30 => 90,
                90 => 180,
                180 => 365,
                _ => 30,
            };
            sync(&ui, plan);
        }
    });
    on!(on_save_plan, |ui, state| {
        let Some(mut plan) = state.draft.clone() else {
            return;
        };
        if state.saved.as_ref() != Some(&plan) {
            plan.reset_review();
        }
        if commit(&ui, state, Some(plan)) {
            detail(&ui, state);
            ui.global::<View>()
                .set_notice("Plan saved. Export an encrypted backup for your heir.".into());
        }
    });
    on!(on_check, |ui, state, index| {
        let Some(item) = state.review.get_mut(index as usize) else {
            return;
        };
        *item = !*item;
        ui.global::<View>()
            .set_checks(ModelRc::new(VecModel::from(state.review.to_vec())));
    });
    on!(on_review, |ui, state| {
        let Some(mut plan) = state.saved.clone() else {
            return;
        };
        plan.checklist = inheritance_core::Checklist {
            keys_located: state.review[0],
            access_arranged: state.review[1],
            backup_shared: state.review[2],
            rehearsed: plan.checklist.rehearsed,
        };
        if let Err(error) = plan.upgrade().and_then(|_| plan.mark_reviewed(now())) {
            fail(&ui, error);
            return;
        }
        if commit(&ui, state, Some(plan)) {
            detail(&ui, state);
            page(&ui, 17);
            ui.global::<View>()
                .set_notice("Review recorded. Your wallet rules are unchanged.".into());
        }
    });
    on!(on_handover_check, |ui, state, index| {
        if let Some(value) = state.handover.get_mut(index as usize) {
            *value = !*value;
            ui.global::<View>()
                .set_handover_checks(ModelRc::new(VecModel::from(state.handover.to_vec())));
        }
    });
    on!(on_handover_save, |ui, state| {
        let Some(mut plan) = state.saved.clone() else {
            return;
        };
        if let Err(e) = plan.upgrade() {
            fail(&ui, e);
            return;
        }
        plan.care.as_mut().unwrap().handover = state.handover;
        if let Err(e) = plan.revise(state.saved.as_ref(), now()) {
            fail(&ui, e);
            return;
        }
        if commit(&ui, state, Some(plan)) {
            detail(&ui, state);
            page(&ui, 17);
        }
    });
    on!(on_next_step, |ui, state| {
        let step = ui.global::<View>().get_guide_step();
        if step >= 4 {
            page(&ui, 15);
        } else {
            guide(&ui, state, step + 1);
        }
    });
    on!(on_export_kit, |ui, state| {
        let v = ui.global::<View>();
        let password = Zeroizing::new(v.get_backup_password().to_string());
        if let Err(e) = validate_backup_password(&password) {
            fail(&ui, e);
            return;
        }
        if password.as_str() != v.get_backup_confirmation().as_str() {
            fail(&ui, "Passwords do not match.");
            return;
        }
        let Some(mut exported) = state.saved.clone() else {
            return;
        };
        if exported.care.is_none() {
            if let Err(e) = exported.revise(state.saved.as_ref(), now()) {
                fail(&ui, e);
                return;
            }
        }
        start_crypto(&ui, state, move || {
            encrypt_backup(&exported, &password).map(|bytes| CryptoResult::Encrypted {
                plan: exported,
                bytes,
            })
        });
    });
    on!(on_import_kit, |ui, state| {
        match pick(false).and_then(|p| {
            p.map(|p| {
                read(&p).and_then(|bytes| {
                    validate_encrypted_backup(&bytes)?;
                    Ok(bytes)
                })
            })
            .transpose()
        }) {
            Ok(Some(bytes)) => {
                state.incoming = Some(bytes);
                page(&ui, 18);
            }
            Ok(None) => {}
            Err(e) => fail(&ui, e),
        }
    });
    on!(on_unlock_backup, |ui, state| {
        let Some(bytes) = state.incoming.clone() else {
            return;
        };
        let password = Zeroizing::new(ui.global::<View>().get_backup_password().to_string());
        start_crypto(&ui, state, move || {
            decrypt_backup(&bytes, &password).map(CryptoResult::Decrypted)
        });
    });
    on!(on_crypto_finished, |ui, state| {
        let result = state.crypto_result.lock().unwrap().take();
        ui.global::<View>().set_crypto_busy(false);
        match result {
            Some(Ok(CryptoResult::Decrypted(plan))) => {
                state.incoming = None;
                page(&ui, 8);
                sync(&ui, &plan);
                state.pending = Some(plan);
                duplicate_view(&ui, state);
            }
            Some(Ok(CryptoResult::Encrypted { mut plan, bytes })) => {
                match pick(true).and_then(|p| p.map(|p| export_to(&bytes, &p)).transpose()) {
                    Ok(Some(name)) => {
                        if let Some(c) = &mut plan.care {
                            c.exported_revision = Some(c.revision);
                            c.encrypted_export = true;
                        }
                        if commit(&ui, state, Some(plan)) {
                            detail(&ui, state);
                            ui.global::<View>().set_notice(
                                format!(
                                    "Encrypted backup saved: {name}. Keep its password separate."
                                )
                                .into(),
                            );
                        }
                    }
                    Ok(None) => {}
                    Err(e) => fail(&ui, e),
                }
            }
            Some(Err(e)) => fail(&ui, e),
            None => fail(&ui, "Backup operation did not complete."),
        }
    });
    on!(on_restore, |ui, state| {
        let Some(mut plan) = state.pending.clone() else {
            return;
        };
        plan.forget_imported_assurances();
        let previous_selection = state.selected;
        state.selected = None;
        if commit(&ui, state, Some(plan)) {
            detail(&ui, state);
            ui.global::<View>()
                .set_notice("Plan restored. Review the details with your heir.".into());
        } else {
            state.selected = previous_selection;
        }
    });
    on!(on_delete, |ui, state| {
        if ui.global::<View>().get_page() != 10 {
            return;
        }
        if commit(&ui, state, None) {
            home(&ui, state);
            ui.global::<View>()
                .set_notice("Plan deleted from this app.".into());
        }
    });
    on!(on_setup_next, |ui, state| {
        let v = ui.global::<View>();
        v.set_setup_step((v.get_setup_step() + 1).min(4));
        if let Some(p) = &state.draft {
            sync(&ui, p);
        }
    });
    on!(on_select_signer, |ui, state, index| {
        if let Some(p) = &state.draft {
            if p.care
                .as_ref()
                .is_some_and(|c| index >= 0 && (index as usize) < c.signers.len())
            {
                ui.global::<View>().set_signer_index(index);
                page(&ui, 14);
                signer_view(&ui, p);
            }
        }
    });
    on!(on_edit_signer, |ui, state, index| {
        if !(0..3).contains(&index) {
            return;
        }
        let Some(p) = &state.draft else { return };
        let v = ui.global::<View>();
        let i = v.get_signer_index() as usize;
        let Some(s) = p.care.as_ref().and_then(|c| c.signers.get(i)) else {
            return;
        };
        state.edit_return = 14;
        page(&ui, 4);
        v.set_field_index(100 + i as i32 * 3 + index);
        v.set_field_multiline(index != 0);
        v.set_edit_value(
            [&s.name, &s.location, &s.access][index as usize]
                .clone()
                .into(),
        );
        // TODO: localize
        v.set_title(
            [
                "Signing Key Name",
                "Custodian or Location",
                "Access Instructions",
            ][index as usize]
                .into(),
        );
        v.set_field_help(["A recognisable name, such as Home Passport. Up to 80 characters.", "Name the custodian or give a minimal location reference, for example: contact the executor. Avoid exact locations unless needed. Up to 240 characters.", "Reference separate access arrangements. Do not enter PINs, passwords or seed words. Keep this plan from becoming a map to every secret. Up to 240 characters."][index as usize].into());
    });
    on!(on_open_existing, |ui, state| {
        if let Some(id) = state.pending.as_ref().and_then(|p| {
            state
                .library
                .entries
                .iter()
                .find(|e| e.plan.same_wallet(p))
                .map(|e| e.id)
        }) {
            let entry = state.library.entries.iter().find(|e| e.id == id).unwrap();
            state.selected = Some(id);
            state.saved = Some(entry.plan.clone());
            detail(&ui, state);
        }
    });
    on!(on_rehearsal_confirm, |ui, state| {
        let step = ui.global::<View>().get_rehearsal_step() as usize;
        if step < 5 {
            state.rehearsal[step] = !state.rehearsal[step];
            rehearsal_view(&ui, state, step as i32);
        }
    });
    on!(on_rehearsal_next, |ui, state| {
        let step = ui.global::<View>().get_rehearsal_step();
        if step < 4 {
            rehearsal_view(&ui, state, step + 1);
            return;
        }
        let Some(mut p) = state.saved.clone() else {
            return;
        };
        if let Err(e) = p.record_practice(state.rehearsal, now()) {
            if !p.completeness().is_empty() {
                page(&ui, 17);
                sync(&ui, &p);
            }
            fail(&ui, e);
            return;
        }
        if commit(&ui, state, Some(p)) {
            detail(&ui, state);
            page(&ui, 17);
            ui.global::<View>()
                .set_notice("Practice result saved, including any unfinished steps. This is not proof of recovery.".into());
        }
    });
}

fn signer_view(ui: &AppWindow, p: &Plan) {
    let v = ui.global::<View>();
    if let Some(s) = p
        .care
        .as_ref()
        .and_then(|c| c.signers.get(v.get_signer_index() as usize))
    {
        v.set_signer_fingerprint(s.fingerprint.clone().into());
        v.set_signer_fields(ModelRc::new(VecModel::from(vec![
            s.name.clone().into(),
            s.location.clone().into(),
            s.access.clone().into(),
        ])));
    }
}

fn duplicate_view(ui: &AppWindow, state: &State) {
    let duplicate = state
        .pending
        .as_ref()
        .and_then(|p| state.library.entries.iter().find(|e| e.plan.same_wallet(p)));
    let v = ui.global::<View>();
    v.set_duplicate(duplicate.is_some());
    v.set_duplicate_name(
        duplicate
            .map(|e| e.plan.wallet_name.clone())
            .unwrap_or_default()
            .into(),
    );
}

fn rehearsal_view(ui: &AppWindow, state: &State, step: i32) {
    let v = ui.global::<View>();
    let cards = state
        .saved
        .as_ref()
        .and_then(|p| p.validate().ok().map(|w| signing_key_cards(p, &w)))
        .unwrap_or_default();
    v.set_rehearsal_key_headings(ModelRc::new(VecModel::from(
        cards
            .iter()
            .map(|(heading, _)| heading.as_str().into())
            .collect::<Vec<crate::slint::SharedString>>(),
    )));
    v.set_rehearsal_key_details(ModelRc::new(VecModel::from(
        cards
            .iter()
            .map(|(_, body)| body.as_str().into())
            .collect::<Vec<crate::slint::SharedString>>(),
    )));
    v.set_rehearsal_step(step);
    v.set_rehearsal_confirmed(state.rehearsal[step as usize]);
    // TODO: localize
    v.set_rehearsal_title(
        [
            "Open the plan together",
            "Identify the signing keys",
            "Check the wallet together",
            "Test password handover",
            "Plan for an unavailable route",
        ][step as usize]
            .into(),
    );
    v.set_rehearsal_body(["On a separate computer, open the encrypted backup and read recovery-guide.txt. Can your heir identify the plan and trusted contact without coaching? Do not put its password in this plan.", "Together, locate enough distinct signing keys for your chosen path using the instructions below. A device and its backup count as one key. Do not enter seeds or move funds.", "Import wallet.bsms from the decrypted backup into compatible wallet software, or open the existing matching wallet. Compare the network and first receive address below. No transaction is needed. This app cannot verify your actions.", "Can your heir obtain the backup password through the separate arrangement, without relying on information inside the locked backup? Never record the password here.", "Suppose the device, one backup copy or your first contact is unavailable. Walk through an alternative route. Extra copies improve availability but increase exposure. Mark this unfinished if no workable alternative exists."][step as usize].into());
}

fn signing_key_cards(p: &Plan, w: &inheritance_core::Wallet) -> Vec<(String, String)> {
    let missing = |s: &str| {
        if s.trim().is_empty() {
            // TODO: localize
            "Not recorded. Ask the owner to add these instructions.".to_string()
        } else {
            s.to_string()
        }
    };
    let mut cards = Vec::new();
    // TODO: localize
    if let Some(policy) = &w.inheritance {
        cards.push(("Normal access".into(), format!("Use {} of {} normal keys. This path remains available before and after the inheritance timelock.", w.threshold, policy.normal_count)));
        cards.push((
            "Inheritance access".into(),
            format!(
                "Use {} of {} inheritance keys once the timelock is satisfied.",
                policy.threshold,
                w.fingerprints.len() - policy.normal_count
            ),
        ));
        let lock = if policy.after < 500_000_000 {
            format!("After block height {}.", policy.after)
        } else {
            format!("Timelock date: {} UTC.\n\nEligibility uses blockchain median time, not this device's clock.", inheritance_core::utc_date(Some(policy.after as u64)))
        };
        cards.push(("When inheritance is available".into(), format!("{lock}\n\nCompatible wallet software must check blockchain eligibility. This app does not unlock funds.")));
    } else {
        cards.push(("Keys required".into(), w.signing_rules()));
    }
    if let Some(care) = &p.care {
        for (index, signer) in care.signers.iter().enumerate() {
            let name = if signer.name.trim().is_empty() {
                w.signer_role(index)
            } else {
                &signer.name
            };
            cards.push((
                name.into(),
                format!(
                    "{}\nFingerprint: {}\n\nLocation\n{}\n\nHow to access\n{}",
                    w.signer_role(index),
                    signer.fingerprint,
                    missing(&signer.location),
                    missing(&signer.access)
                ),
            ));
        }
    } else {
        cards.push(("Key fingerprints".into(), w.fingerprints.join("\n")));
    }
    if !p.key_guidance.trim().is_empty() {
        cards.push(("Additional key instructions".into(), p.key_guidance.clone()));
    }
    cards
}

fn guide(ui: &AppWindow, state: &State, step: i32) {
    let Some(p) = &state.saved else { return };
    let Ok(w) = p.validate() else { return };
    let missing = |s: &str| {
        if s.trim().is_empty() {
            "Not recorded. Contact the trusted person for help; do not guess.".to_string()
        } else {
            s.to_string()
        }
    };
    // TODO: localize
    let title = [
        "Start here",
        "Find the signing keys",
        "Additional instructions",
        "Open the wallet",
        "Before you rely on this plan",
    ][step as usize];
    let v = ui.global::<View>();
    v.set_guide_step(step);
    v.set_guide_title(title.into());
    let mut cards: Vec<(String, String)> = Vec::new();
    // TODO: localize
    match step {
        0 => {
            cards.push((
                "Your guidance".into(),
                format!("For {}", missing(&p.heir)),
            ));
            if !p.message.trim().is_empty() {
                cards.push(("Personal message".into(), p.message.clone()));
            }
            cards.push(("Someone who can help".into(), missing(&p.contact)));
        }
        1 => cards = signing_key_cards(p, &w),
        2 => cards.push(("Additional instructions (optional)".into(), if p.access_guidance.trim().is_empty() { "No additional instructions. Use the location and access details recorded for each signing key.".into() } else { p.access_guidance.clone() })),
        3 => {
            cards.push((
                "Open your plan".into(),
                "Open the recovery ZIP with an AES-compatible archive tool and its separate password. Read recovery-guide.txt. You can also restore the ZIP in Inheritance.".into(),
            ));
            cards.push(("Open compatible wallet software".into(), "Import wallet.bsms from the decrypted archive, or open your existing matching wallet. The encrypted ZIP itself is not a wallet file. Your wallet software must support the complete policy.".into()));
            cards.push(("Compare wallet details".into(), format!("Network: {}\n\nFirst receive address\n{}\n\nCheck these match before continuing.", w.network, w.first_address)));
        }
        _ => {
            cards.push(("Next steps".into(), "Use the required signing keys and follow the recovery instructions for your wallet software. If a key or instruction is missing, stop and contact the trusted person.".into()));
            cards.push(("Check the wallet".into(), "Confirm the wallet details match your trusted wallet software. Check the policy, network and first receive address before using the wallet. This app cannot confirm recovery will succeed.".into()));
            cards.push((
                "Keep secrets separate".into(),
                "Never enter seed words on a website or share them with support.".into(),
            ));
        }
    }
    v.set_guide_headings(ModelRc::new(VecModel::from(
        cards
            .iter()
            .map(|(heading, _)| heading.clone().into())
            .collect::<Vec<_>>(),
    )));
    v.set_guide_details(ModelRc::new(VecModel::from(
        cards
            .into_iter()
            .map(|(_, detail)| detail.into())
            .collect::<Vec<_>>(),
    )));
}

struct Selected {
    path: String,
    location: fs::Location,
}
fn start_crypto(
    ui: &AppWindow,
    state: &mut State,
    work: impl FnOnce() -> Result<CryptoResult, String> + Send + 'static,
) {
    let v = ui.global::<View>();
    v.set_backup_password("".into());
    v.set_backup_confirmation("".into());
    v.set_crypto_busy(true);
    if state.crypto_sender.is_none() {
        let result = state.crypto_result.clone();
        let weak = ui.as_weak();
        let (tx, rx) = mpsc::channel::<CryptoWork>();
        if std::thread::Builder::new()
            .name("backup-crypto".into())
            .spawn(move || {
                for work in rx {
                    *result.lock().unwrap() = Some(work());
                    let _ = weak.upgrade_in_event_loop(|ui| {
                        ui.global::<Actions>().invoke_crypto_finished()
                    });
                }
            })
            .is_err()
        {
            v.set_crypto_busy(false);
            fail(ui, "Could not start password protection. Please try again.");
            return;
        }
        state.crypto_sender = Some(tx);
    }
    if state
        .crypto_sender
        .as_ref()
        .unwrap()
        .send(Box::new(work))
        .is_err()
    {
        v.set_crypto_busy(false);
        state.crypto_sender = None;
        fail(ui, "Could not start password protection. Please try again.");
    }
}
fn pick(folder: bool) -> Result<Option<Selected>, String> {
    let options = SelectFileOptions::default()
        .with_allowed_locations(AllowedLocations::All)
        .with_allowed_extensions(AllowedExtensions::All)
        .with_dirs_allowed(true)
        .with_hidden_allowed(false)
        .with_dir_selection_mode(folder)
        .with_multiple_selection_mode(false);
    let result = select_file::<GuiPermissions>(options)
        .map_err(|_| "Could not open the file picker.".to_string())?;
    Ok(result
        .and_then(|r| r.files().first().cloned())
        .map(|(path, location)| Selected {
            path,
            location: match location {
                Location::Internal => fs::Location::User,
                Location::External => fs::Location::Usb,
                Location::Airlock => fs::Location::Airlock,
            },
        }))
}
fn read(selected: &Selected) -> Result<Vec<u8>, String> {
    let file = FileSystem::default()
        .open_file(&selected.path, selected.location, fs::OpenFlags::READ_ONLY)
        .map_err(|_| "Could not open the selected file.".to_string())?;
    let mut bytes = Vec::new();
    file.take((MAX_BACKUP + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| "Could not read the file. Reconnect the drive and try again.".to_string())?;
    if bytes.len() > MAX_BACKUP {
        return Err("The selected file exceeds the backup size limit.".into());
    }
    Ok(bytes)
}
fn scan() -> Result<Option<Vec<u8>>, String> {
    let result = open_qr_scanner::<GuiPermissions>(ScanQrOptions {
        header_title: "Scan Wallet Export".into(),
        ..ScanQrOptions::default()
    })
    .map_err(|_| "Could not open the QR scanner.".to_string())?;
    match result {
        Some(ScanQrResult::Qr { data, .. }) => Ok(Some(data)),
        Some(ScanQrResult::Ur2 { .. }) => {
            Err("Choose a BSMS file instead. Animated UR wallet exports are not supported.".into())
        }
        _ => Ok(None),
    }
}
fn export_to(bytes: &[u8], folder: &Selected) -> Result<String, String> {
    validate_encrypted_backup(bytes)?;
    let fs = FileSystem::default();
    for suffix in 0..100 {
        let name = format!("inheritance-{}-{suffix}.zip", now());
        let prefix = format!("{}/{}", folder.path.trim_end_matches('/'), name);
        let paths = [prefix];
        let mut collision = false;
        for path in &paths {
            match fs.open_file(path, folder.location, fs::OpenFlags::READ_ONLY) {
                Ok(_) => collision = true,
                Err(fs::Error::FileNotFound) => {}
                Err(_) => return Err("Could not access the export folder.".into()),
            }
        }
        if collision {
            continue;
        }
        for path in &paths {
            let mut file = fs
                .open_file(path, folder.location, fs::OpenFlags::CREATE)
                .map_err(|_| "Could not create the backup. Check the drive.".to_string())?;
            file.write_all(bytes)
                .and_then(|_| file.flush())
                .map_err(|_| {
                    "Export was interrupted. Partial files may remain; export again.".to_string()
                })?;
            drop(file);
            let copy = read(&Selected {
                path: path.clone(),
                location: folder.location,
            })?;
            if copy != bytes {
                return Err("Export verification failed. Export again to a trusted drive.".into());
            }
        }
        return Ok(name);
    }
    Err("Choose another export folder.".into())
}
