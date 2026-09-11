# Validation and device acceptance

## v0.6.1 vendor-neutral wallet guidance

- Removed wallet-vendor-specific positioning from the app, recovery guides,
  fixtures and documentation. BSMS remains the standard interchange format.
- Copy names the supported policy boundaries and tells users to compare the
  complete policy, network and first receive address in compatible wallet software.
- All 81 core tests, 17 UI contract checks and 116 offscreen layout
  renders passed. The app also compiled and ran its test target in release mode.
- No simulator or physical device was used for this copy-only update. Real
  cross-wallet BSMS import remains a device/user acceptance check.

## v0.6.0 safety and handover

- 81 core tests and 24 package/UI checks passed, including old-field migration,
  incomplete practice, independent review dates and assurance resets.
- Independent AES ZIP tests passed in both directions with pyzipper and macOS
  libarchive. New exports use the standalone heir guide; older guide serialization
  remains supported for legacy backup validation.
- Rendered 20 pages plus setup/practice/long-text/menu variants at two heights
  in both themes. Offscreen viewer does not render embedded SVG icons.
- One simulator was launched. Checked partial handover persistence, stale backup
  receipt, full key-detail scrolling, skipped practice saved as 0/5, and the
  five-confirmation save route. The debug channel subsequently closed; no second
  simulator was launched. Physical-device and outsider acceptance remain pending.
- QnA-signed 0.6.0 package validated for Beta 3 and copied byte-for-byte to SD.
  See DEVICE-SIGNOFF-0.6.0.md for remaining device/interoperability/outsider checks.

## v0.5.1 optional additional instructions

- General additional instructions are optional in completeness, review and
  rehearsal checks. Individual signing-key location/access requirements remain.
- Existing notes and the saved JSON field are preserved, including encrypted
  backup round trips. Setup, editor, guidance and exported guide use the new label.
- Regression tests cover blank optional notes, successful review/rehearsal,
  retained existing notes and missing required signing-key instructions.
- All 78 core tests and 21 package/UI checks passed. The user's SD-card ZIP
  validates with zero missing required details using the new core, without
  extracting plaintext or modifying the archive. Setup layout inspected offscreen;
  no simulator was opened and no hardware was driven.

## v0.5.0 portable encrypted ZIPs

- Standard AES-256 ZIP, three encrypted members, no plaintext export path.
- Independent pyzipper decrypt/encrypt and macOS libarchive decryption agree.
  Run `tests/check_zip_interop.py` in the isolated ZIP test environment.
- Regression checks cover fresh salts, per-member authentication, wrong passwords,
  malformed/extra/plaintext members, invalid counts and legacy encrypted restore.
- Includes guidance cards, explicit rehearsal blockers, fixed confirmation footer
  and fresh, non-persisted review confirmations. Device acceptance remains in
  `ENCRYPTED-BACKUP.md`; older format descriptions below are historical.
- 77 core tests and 20 package/UI checks passed, plus authored-code Clippy,
  native and signed hardware builds. Final app hash:
  `cb2d042d9956f4b6ea91d6b24857c7f9307dbe09b107249d1c03d553ed0c04e3`.
- Simulator exported a ZIP through the file picker, read it back, unlocked it,
  detected the existing wallet and restored a separate plan with reset assurances.
  The long-running SDK kernel hit its known thread-index panic before this test;
  all old processes were confirmed gone before one fresh simulator was launched.
- Six independently constructed invalid ZIPs (altered guide/wallet, extra member,
  plaintext member, compression and archive comment) were rejected.
- Password page layout inspected at 480x760; inputs and fixed action fit. No
  physical device was driven or flashed. Real wallet-software import remains untested.

## v0.4.0 encrypted-only backups

- Password backups use Argon2id and XChaCha20-Poly1305. Independent libsodium
  derivation/decryption and encryption/Rust restore agree in both directions
  (`tests/check_crypto_interop.py`, public demo data only).
- Five encryption regressions cover round trips, fresh randomness, absence of
  plaintext fields, wrong passwords, header/ciphertext/tag tampering, truncation,
  appended data, plaintext/unknown-version rejection, size/password limits and
  older plaintext export receipts. Total: 73 core tests.
- Simulator checked password masking, mismatched confirmation, encrypted file
  creation/read-back, plaintext rejection and wrong-password input clearing.
- Correct-password restore reached duplicate detection and restored a separate
  plan without overwriting the original; the restored plan reset its export
  receipt and revision. Keyboard Done dismissed the password keyboard.
- 17 package/UI checks passed. Eight password-screen layout variants rendered
  at 480x760 and 480x800 in both themes; representative export and unlock
  screens were visually inspected. The final hardware package is QnA-signed.
- The SDK kernel again hit its process.rs:252 thread-index error during extended
  input testing. All processes had exited before restarting one simulator.
- See `ENCRYPTED-BACKUP.md` for the current device acceptance checklist. Historical
  plaintext-export instructions below do not apply to v0.4.

## v0.3.1 BSMS compatibility

- Tested both user-supplied `Aug 27 Test` exports directly from SD, without
  copying public keys into repository fixtures or changing either original.
- Both describe the same normal 2-of-3 / timelocked inheritance 1-of-3 policy.
- Compared 100 receive and 100 change scripts per source against the restored
  recovery kit; all matched. BSMS first-address validation and both round trips
  passed. This is not a transaction-signing or wallet-software re-import test.
- Added seven synthetic policy regressions: separate paths, absolute time/height,
  unsupported relative locks, BSMS address/checksum validation, invalid template
  restrictions, reordered signer-card rejection, and per-path completeness with
  recovery-kit round trips. All 68 core tests and 14 package/UI checks passed.
- Eligibility is never inferred from the device clock. Actual spending and
  hardware signer support must still be checked in compatible wallet software.

## v0.3.0 delivery validation

- 61 Rust core tests (17 new structured-plan tests plus 44 existing tests):
  legacy migrations, key/identity validation, Unicode limits including five
  signers and 20 maximum-size plans, missing details, revision/no-op/export
  transitions, malformed kits, restore assurances and rehearsal confirmations.
- 14 Python package/UI contract checks. Both signed objects and all manifest
  asset hashes verified; minimum firmware 1.4.0-beta3 retained.
- Hardware release and hosted builds, authored-code Clippy and rustfmt checks.
- 116 offscreen layout variants: 18 pages, setup/rehearsal stages, both themes,
  480x760 and 480x800, long/empty/menu cases. The bundled interpreter cannot load
  SVG images; embedded guidance icons were checked in the compiled simulator
  instead. Offscreen rendering is not an assertion that every pixel was inspected.
- Simulator: migrated existing plans; filled two fingerprint-linked signer
  cards through the keyboard; completed/resumed setup; recorded rehearsal only
  after confirmations; verified export receipt and persistence after app restart;
  renamed a signer and observed a new revision/stale export; opened an existing
  duplicate without adding a plan; restored an actual v2 export separately.
- Extended simulator use encountered a hosted-kernel panic at
  `xous/kernel/src/process.rs:252` (thread-priority array index 32, length 32).
  The leftover GUI processes were confirmed absent before one replacement
  launch. This is a simulator limitation observed during the run, not evidence
  that the app has passed equivalent physical-device testing.

Use `ACCEPTANCE.md` for the current device checklist. Notes below are historical
v0.2/v0.1 evidence, not the v0.3 navigation contract. No real device was flashed
or driven during this delivery. No certificate was generated or rotated.

Internal POC v0.2.0, Foundation SDK 1.0.0, 8 September 2026.

## v0.2 multi-plan and UI regression checks

- 43 core tests: the 31 baseline tests plus 12 collection tests. These cover
  legacy migration, deleted legacy data, multi-plan round trips, stable IDs,
  isolated edits/reviews/deletion, restore-as-new, failed mutations, capacity
  limits, corrupt collections and maximum-length notes across 20 plans.
- 12 Python checks: seven release-package/permission tests plus five UI
  contract checks for native SDK menu rows, one card per plan, SDK typography,
  top anchoring, keyboard exit, primary CTA and valid SDK icon names.
- 68 offscreen variants cover 12 screens at both supported test sizes/themes,
  plus long plan names, long fields, menus and empty states. Icons are also
  checked in the running simulator, since preview callbacks do not load them.
- Simulator checks: existing 0.1 plan retained, second plan added by restore,
  second name edited without changing the first, independent review dates,
  selected-plan export, cancelled deletion retaining both plans, and deletion
  retaining the other plan and its review state. Restoring another plan leaves
  the already reviewed plan unchanged.
- Final-build checks: a third plan created through BSMS import and keyboard
  entry, then deleted without affecting either existing plan; all five guidance
  steps rendered with SDK icons; ten consecutive menu open/dismiss cycles
  produced identical open-menu screenshots. Restart retained both plans and
  their independent review states.
- The simulator process/window was closed and its absence verified before
  each replacement launch. Only one instance is left running.

The baseline notes below describe the initial version's validation. For v0.2,
restoring **adds** a plan; it no longer replaces one. Export remains per-plan.

## Completed

- **31 Rust core tests passed**, including public descriptor/BSMS parsing,
  the BIP-129 published public-wallet vector, expansion of receiving/change
  branches, invalid checksums/addresses, mixed networks, duplicate signers,
  actual private-key rejection, receive-only rejection, unknown schemas,
  oversized/truncated data, Unicode boundaries, round trips, clock rollback
  and review expiry. Includes a 1,024-input malformed-byte smoke test, not a
  comprehensive fuzzer or security audit.
- **7 release-package tests passed**: valid package, changed binary, missing
  minimum firmware, wrong app identity, extra files, missing signature header,
  and permission allowlist. `cosign2 dump` independently verified both signed
  objects during packaging; all bundled asset hashes were validated.
- Targeted Rust formatting passed. Core/all-target Clippy and app Clippy with
  `--no-deps -D warnings` passed. SDK dependency and non-Window Slint export
  warnings remain upstream; authored Rust has no Clippy warnings.
- **60 offscreen UI variants rendered**: 11 screens in both themes at
  480×760 and 480×800, plus menu, empty state and long-text cases. Representative
  screens were visually inspected; longer forms/guides intentionally scroll.
- Simulator: native file picker and permission prompts, BSMS import/address
  review, keyboard entry, header Done/keyboard dismissal, save, restore
  cancellation, restore replacement, incomplete-review rejection, all four
  checklist controls, review recording, all five guide steps, and export.
- Simulator export produced guide, JSON kit and BSMS files; each was flushed
  and read back by the app. Re-importing that exported JSON succeeded and reset
  the checklist. Closing/relaunching retained the saved plan/review status.
- Delete cancellation retained the plan. Confirmed deletion remained deleted
  after closing/relaunching. Demo data only; no hardware seeds were used.
- Final ARM release built, signed with the existing `qna-dev` developer identity,
  and validated with minimum firmware `1.4.0-beta3`.

SDK capture colours have swapped red/blue channels in this simulator build.
Use the live simulator or offscreen previews for palette assessment. Screenshots
are under `target/sim-*.png` and `target/previews/`; they have no device frame.

## Reproduce

```sh
cargo test -p inheritance-core
cargo clippy -p inheritance-core --all-targets -- -D warnings
cargo fmt -p keyos-inheritance -p inheritance-core -- --check
python3 -m unittest discover -s tests -p 'test_*.py'
bash scripts/check-ui.sh
```

After `foundation preview` has staged SDK resources, app Clippy uses:

```sh
FOUNDATION_UI_LIBRARY_PATH=target/foundation/ui/ui \
FOUNDATION_THEMES_SLINT_DIR=target/foundation/themes/slint \
cargo clippy -p keyos-inheritance -p inheritance-core --no-deps -- -D warnings
```

`tests/sim-*.json` records the UI exercise sequences, including screenshots.
They are state-dependent, not assertion-based unattended tests. The native
picker must contain `inheritance-demo` with the supplied fixtures; initial
permission prompts must be approved. Never run them on production plans.
Stop/close the existing simulator before starting another.

## Check on Passport Prime

Use test data first. Never fund the included demo wallet.

1. Install `Inheritance-0.2.0-beta3.app` using the existing QnA developer
   certificate. Open the app and try the menu repeatedly, including dismissal.
2. Choose **Create a Plan → Choose Wallet File**, then `demo-wallet.bsms`.
   Expect **2 of 3**, **Test network**, fingerprints **4BA43603, 8DFC9B34,
   56C4FAC3**, and an address beginning `tb1qzvy8`.
3. Add a wallet name and heir. Try typing a long key-location note, scrolling,
   using header Done, and backing out of a field. Save and reopen the app.
4. Alternatively choose **Plans → … → Restore Plan → demo-plan.json**.
   Review and confirm. Expect an additional **Family Vault (demo)** / **Alex Rivera**
   card. Open it, change its name and check the other plan stays unchanged.
5. Open Plan Review. Try recording without checks: it must refuse. Confirm the
   first three items, record, and expect **Review in 90 days**. Edit and save a
   field: review date/checks must reset. Nothing should contact external wallet software.
6. Read all five Heir Guidance steps. Use the Plan Details menu to export to
   the SD/USB drive. Expect three
   new files sharing a prefix. Open the guide on a computer; restore the JSON
   on the device. Compare the wallet details before adding the restored plan.
7. Verify the BSMS wallet file imports into compatible wallet software and
   both receive/change branches match. This cross-app hardware acceptance is
   **not yet verified**; do not use the POC as your only recovery method.
8. Cancel deletion, then confirm deletion. Reopen the app: expect the welcome
   screen only after the last plan is deleted. Other plans and exported files
   must still exist.

Still requires physical verification: installation, small touch targets,
long keyboard editing, real USB/SD removal and full-disk errors, camera/text-QR
scanning, theme changes, and wallet-software interoperability with a real public export.
Power-failure behaviour relies on the SDK durable-file guarantees; no fault
injection or hardware power-cut testing has been performed.

## Sensible next improvements

- Import actual public exports from several compatible wallet applications and broaden the
  interoperability fixture corpus before adding further wallet types.
- Add animated UR wallet-configuration import if the SDK's decoded types can
  be mapped without losing derivation or policy information.
- Consider a distinct wallet badge or user-selected colour for each plan once
  the multi-plan workflow has passed physical-device testing.

Do not add automatic unlock timers or imply support for provider-managed inheritance claims
without the relevant policy/API integration and a separate security review.
