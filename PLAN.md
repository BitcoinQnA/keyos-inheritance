# Inheritance companion v0.3

## v0.3 delivery plan

1. Add bounded, fingerprint-linked signer cards without losing legacy notes.
2. Add resumable guided setup: wallet, signers, people, access, completeness.
3. Separate heir instructions from editing; add an explicit recovery entry and
   a three-stage rehearsal with user confirmations, never a recovery guarantee.
4. Add plan revisions, revision dates, verified-export freshness, and safer
   duplicate-wallet detection with an inspect-existing path.
5. Export a versioned kit with signer instructions and a practical checklist;
   continue importing v1 kits and existing single/multiple-plan storage.
6. Test migration, limits, invalid data, identity matching, revision transitions,
   restore, rehearsal, UI wrapping and navigation, then sign with qna-dev and
   validate the device archive. Preserve older Desktop installers.
7. Before a simulator launch, check kernel, simulator AND gui-server processes.
   Reuse an existing instance; close and verify all three before replacement.

No new network, seed, signing or hardware permissions. Physical recovery and
wallet-software interoperability remain device acceptance tests, not automated claims.

Build a standalone Foundation SDK 1.0.0 app for organising recovery instructions
around supported public multisig wallet configurations. This is an internal POC,
not a wallet or inheritance service.

## Product contract

- Import and validate a public native-SegWit sorted multisig descriptor or BSMS
  wallet configuration. Show threshold, network and signer fingerprints.
- Up to 20 independently saved plans: wallet name, heir, contact, message, key-location guidance and
  instructions for obtaining access. Never request a seed, private key or PIN.
- A review interval and last-reviewed date are organisational reminders only.
  They do not release information, send notifications or change wallet rules.
- Provide a readiness checklist and a step-by-step rehearsal for an heir.
- Save in the app's encrypted device storage. Export an explicitly unencrypted,
  recovery kit (text guide, BSMS wallet file and JSON restore data) to a selected folder. Import that kit on a
  different device with validation and an explicit add-plan confirmation.
- Confirm deletion of the selected plan; cancel must preserve every plan.
- Migrate legacy single-plan storage without losing any fields or review state.
- No master-seed access, signing, online wallet API, automatic claims or
  secure-element proof-of-life claims. Advanced Miniscript policies are rejected
  rather than described as ordinary multisig.

## Implementation

1. Pure Rust core: bounded descriptor/BSMS parsing, plan validation, versioned
   serialization, review-date arithmetic, readiness and readable instructions.
2. SDK integration: native file picker, persistent storage, keyboard and themes.
3. Short, scrollable screens: Plans, Plan Details, import review, plan fields, checklist,
   heir guide, export warning, restore review, about and deletion confirmation.
4. Standard SDK menus/cards, token-based typography, guidance icons, top-aligned
   content, bottom primary actions, large back targets and both colour schemes.

## Verification before handoff

- Parser: real generated xpub fixtures; BSMS checksum and first-address matching;
  malformed, private, oversized, duplicate and mixed-network inputs; unsupported
  scripts and mismatched backup metadata.
- State: round trips; unknown versions; validation limits; cancellation and
  restore-as-new; isolated edits/deletion; legacy migration; capacity limits;
  date boundaries and clock rollback; stale checklist after edits.
- UI: compile and render all screens at 480x760 and 480x800 in both themes;
  inspect long-content and populated states.
- Runtime: simulator import, editing, save/restart, review, export/restore,
  cancellation and deletion using public demo data only.
- Build: format, core tests, app compilation/lints, hardware release package;
  verify signed manifest, hashes and minimum firmware metadata.
- Document supported inputs, limitations, reproducible steps and remaining
  physical-device acceptance checks. Leave the installer on the Desktop.
