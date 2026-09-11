# Inheritance

An **internal proof of concept** built as a standalone Foundation SDK app for
Passport Prime. It organises recovery instructions for an existing, ordinary
Nunchuk multisig wallet. It is inspired by the Nunchuk Inheritance showcase,
but is a new implementation, not a port of that prototype or an official
Nunchuk product.

## What it does

1. Import a public BSMS wallet file or checksummed multipath descriptor.
2. Compare the required signatures, network, fingerprints and first address
   against your wallet in Nunchuk.
3. Follow guided setup: name the wallet, describe each fingerprint-linked
   signing key, name your heir/contact, then record access instructions.
   Save incomplete work and continue later. Save up to 20 independently named plans.
4. Use **Check Your Plan** to review details and optionally practise with your heir.
   Review and practice have separate dates. Heir Guidance provides five recovery steps.
5. Export one password-protected ZIP containing a readable `.txt` guide, a `.bsms`
   wallet configuration and a `.json` saved plan. No plaintext companion files are
   exported. Arrange access to the backup and its separately stored password.

Completeness counts distinct keys with both location and access instructions;
it does not prove key possession or recoverability. Each signing key has a friendly
name, immutable imported fingerprint, location and access instructions. Legacy
free-text key notes remain available as additional notes, never guessed into cards.

Each saved change has a revision and UTC date (or an explicit unknown-date state).
Only a fully written and read-back-verified export is marked current. Changes
make that receipt stale; use **Check Your Plan → Export Updated Backup**. The
receipt does not establish that the heir still has or can read that external copy.

The review interval (30, 90, 180 or 365 days) is only an in-app reminder.
It does not send notifications, release information or unlock bitcoin.
Editing a plan or restoring a kit resets the review checklist/date.
Only the edited/restored plan is affected. Restoring adds a new plan, even if
it describes the same wallet; it never replaces an existing plan. Duplicate
wallet imports warn and offer **Open Existing Plan** or an explicit separate
plan. Matching uses the validated first receive address and network, not its
friendly name or short ID. Restores discard imported export/rehearsal assurances.

The Plans screen has one SDK card per plan. Open a card for its details,
editing, completeness and rehearsal. **Heir Guidance** is the bottom primary action,
leading to **Open Recovery Guide**. This is a simplified view, not a lock or access
restriction. Practice records five individual checks: opening the backup on a
computer, finding signing keys, matching the wallet in Nunchuk, obtaining the
password separately and testing an alternative access route. Unfinished steps
can be skipped and saved without claiming completion. Review has a separate date;
it never contacts Nunchuk or checks devices automatically.
The SDK overflow menu offers Export and Delete for the selected plan; the
Plans menu offers Restore and About. Guidance uses top-aligned, scrollable
steps with centred, embedded vector icons and teal step headers, and SDK text-size tokens.

Upgrading from 0.1.0 automatically retains its saved plan. Collection storage
uses a new schema; export kits before downgrading because 0.1.0 cannot read it.
Version 0.4 retains locally saved plans, but only accepts password-encrypted
backups through Restore. Plaintext JSON backups from earlier versions are not
accepted. Do not downgrade after updating your plans.

## Security boundaries

**This SDK app cannot access the device's master seed or master key.** It has
no seed-vault or signing permission, cannot sign transactions, and never
imports a seed automatically. Wallet import accepts public configurations
only; it rejects private keys and seed phrases. Do not put secrets in the
free-text notes: they are not a secret-detection mechanism.

This is an organiser, not a wallet or inheritance service. It cannot create,
activate or claim a Nunchuk service plan, provide proof of life, contact an
executor, or change the wallet's spending conditions. Recovery still needs
the required signing keys and access to the associated devices/backups.
It does not establish that those keys exist or are usable.

The plan is saved using SDK durable storage in encrypted app data. Export writes
**one AES-256-encrypted ZIP**, containing a readable recovery guide, wallet.bsms
for Nunchuk, and plan.json for app restore. No plaintext sidecars or temporary
plaintext files are written. Open on a computer using an AES-ZIP-compatible
archive tool, or restore directly in Inheritance. Each member has a fresh salt
from the SDK's getrandom 0.2 hardware TRNG route. Encryption runs off the UI thread.

Use five or more random words (minimum 20 characters, maximum 256 UTF-8 bytes),
not the device PIN. Passwords are case-sensitive and spaces are preserved.
There is **no reset or recovery without the password**. Arrange separate access
for the heir. Desktop extraction creates readable files: use a trusted computer.
Import the decrypted wallet.bsms into Nunchuk, not the encrypted ZIP itself.
The standard ZIP password derivation is weaker against offline guessing than
the older Argon2id format. See [format and test instructions](ENCRYPTED-BACKUP.md)
for the trade-off and compatibility limits. Older encrypted .inheritance files
remain restorable; new exports are ZIP only.

The password is not persisted. UI fields are cleared after submission/cancel;
owned password, derived-key and plaintext buffers use zeroizing wrappers.
Slint's immutable strings may have transient copies; this is not a guarantee
that every in-memory copy is immediately erased. The protection also depends
on password strength and device/SDK security. This POC has not had a security audit.

Only ciphertext is written, flushed and read back before success is recorded.
Interrupted exports can leave incomplete encrypted files, never plaintext
sidecars. Encryption does not conceal file size or export time. Older plaintext
exports are not encrypted or deleted by upgrading; they remain readable.

Deleting the plan does not delete exported copies, affect the wallet, or
guarantee forensic erasure of storage blocks. Losing this device or its unlock
credentials can make the local plan inaccessible. Do not make it the only copy.

## Supported wallets and limits

- Native SegWit `wsh(sortedmulti(...))`, 2–5 signers, at least 2 signatures.
- Nunchuk `wsh(or_d(multi(...),and_v(v:multi(...),after(...))))` inheritance
  policies: 2–5 normal keys, 1–5 inheritance keys, separate thresholds and an
  absolute time/block-height condition. Other Miniscript shapes fail closed.
  Guidance shows both paths; completeness requires instructions for each path.
  Time eligibility depends on blockchain median time, not the device clock
  ([BIP-113](https://github.com/bitcoin/bips/blob/master/bip-0113.mediawiki)).
- Each signer needs a distinct public extended key and origin fingerprint.
- Both receiving and change branches (`/<0;1>/*`) are required. Receive-only
  descriptors are rejected to avoid an incomplete recovery wallet.
- Unencrypted four-line BSMS 1.0, with `/0/*,/1/*` restrictions and matching
  first receive address, or `No path restrictions` with explicit receive/change
  branches. BIP-129 `/**` templates require explicit restrictions.
  Supplied checksums are verified; raw descriptors require a checksum.
- Bitcoin mainnet and testnet/signet `tb1` addresses. Test-family xpubs cannot
  distinguish testnet from signet. Regtest BSMS addresses are not supported.
- Text QR wallet imports only. Animated UR exports require the file route.
- Up to 20 saved plans; 80-character names, 200-character contact, 1600-character
  general notes, 240-character signer location/access fields, 8 KiB wallet input
  and 128 KiB per ZIP member (397312 bytes per encrypted archive). A device and
  its backup are one signing key.

Unsupported: singlesig, Taproot, other script wrappers, other Miniscript policies,
Nunchuk autonomous/service inheritance, encrypted BSMS, proprietary wallet
backups, balance lookup, transaction signing and automatic claims.

## Build and test

Requires Foundation SDK **1.0.0** and its Nix/toolchain environment. Dependencies
come from the installed public SDK, not a separate KeyOS worktree. The generated
`.foundation-sdk` symlink is local/ignored. `Cargo.lock` pins Rust dependencies.

```sh
foundation doctor
cargo test -p inheritance-core
cargo clippy -p inheritance-core --all-targets -- -D warnings
cargo fmt -p keyos-inheritance -p inheritance-core -- --check
foundation preview
bash scripts/check-ui.sh
foundation sim
```

Close the existing simulator before starting another. Only synthetic public
fixtures under `tests/fixtures` should be used for demos. Their keys are public
test data derived from known seeds: **never fund these wallets**.

For a device release, enter the SDK Nix shell, then:

```sh
foundation pack --release --out target/inheritance-sdk.app
python3 scripts/pack-beta3.py target/inheritance-sdk.app target/Inheritance-0.6.0-beta3.app
```

The second step verifies signatures and file hashes and preserves Beta 3's
required minimum-firmware field when the CLI omits it. It uses the existing
`qna-dev` signing identity; no developer secret is included in this repository.
Other developers must configure their own identity and developer certificate.

Minimum firmware: **1.4.0-beta3**. Copy the final `.app` to a drive and install
through Settings → Apps after adding the matching developer certificate.
No Foundation production signature is required. Physical-device acceptance
remains necessary; simulator checks cannot establish touch, USB or camera
behaviour on hardware.

## References

- [Foundation SDK](https://docs.foundation.xyz/developers/get-started/)
- [Original showcase](https://foundation.xyz/app-showcase/nunchuk-inheritance)
- [BSMS / BIP-129](https://github.com/bitcoin/bips/blob/master/bip-0129.mediawiki)
- [Nunchuk personal-wallet recovery](https://resources.nunchuk.io/wallet-recovery/personal-wallet/)
- [Nunchuk inheritance claims — a separate service](https://nunchuk.io/howtoclaim)

See `PLAN.md` for the implementation plan and `TESTING.md` for validation
evidence and device-test steps. This POC is not independently security audited.
