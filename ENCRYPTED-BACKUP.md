# Portable encrypted backups (Inheritance 0.5.0)

## What is exported

One AES-256-encrypted ZIP containing exactly three files:

- `recovery-guide.txt`: readable recovery instructions and wallet details.
- `wallet.bsms`: standard public wallet configuration for compatible software.
- `plan.json`: structured plan for restoring in Inheritance.

All three file contents are encrypted. The app builds and encrypts in memory,
then writes only the completed encrypted ZIP. No plaintext companion or temporary
file is written. Generic filenames, sizes and ZIP metadata are not encrypted.
Never put seed words, device PINs or other secrets into the plan.

## Opening on a computer

Use an AES-ZIP-compatible archive tool, such as 7-Zip or WinZip. Do not assume
the operating system's default ZIP opener supports AES encryption.

1. Open the ZIP and enter the separate export password.
2. Read `recovery-guide.txt` in any text editor.
3. Import `wallet.bsms` into compatible wallet software, or open the matching existing wallet.
4. Compare the network, first address and signing policy before proceeding.

The ZIP itself is not a wallet file. The heir still needs
the actual signing keys; this guide cannot unlock funds or establish timelock
eligibility. Compatible wallet software must determine that from the blockchain.

macOS libarchive decryption and Python pyzipper interoperability were tested.
GUI archive applications and real wallet-software re-import remain device/user acceptance
checks, not claims of completed testing.

**Deliberate extraction creates readable files on that computer.** Use a trusted
computer and a non-synced destination; ordinary deletion does not guarantee
forensic erasure. Do not upload this archive or its password to an online opener.
The Inheritance app can restore the ZIP directly without extracting to storage.

## Passwords and security trade-off

Use five or more randomly chosen words and at least 20 characters (maximum
256 UTF-8 bytes). Length alone does not guarantee strength. Passwords are
case-sensitive; spaces and Unicode bytes are preserved. Do not use the device PIN.

WinZip AES uses PBKDF2-HMAC-SHA1 with 1,000 iterations, AES-256 CTR and a 10-byte
HMAC-SHA1 authentication tag per member (AE-1 or AE-2). This standard is more
portable but substantially cheaper to attack offline than the older Argon2id
format. There is no reset or password recovery. Keep the password separate.

Encryption uses the zip crate with a documented getrandom 0.2 adaptation for the
SDK's hardware TRNG. Compression features are disabled. Each member gets a
fresh random 16-byte salt. The archive's authenticated plan envelope (format 2)
includes SHA-256 hashes binding the guide and wallet contents to the plan.

Restore accepts exactly the three expected members in order, each encrypted
using AES-256 and stored without compression. It bounds file sizes (128 KiB
per member; 397312 bytes overall), rejects plaintext, ZipCrypto, other members,
paths, multi-volume archives, ZIP64, archive comments and unsupported versions.
It authenticates all members before parsing the plan and checks wallet equality.
Modified/repacked archives may not meet this deliberately restricted format.

Only a flushed and read-back-matching ZIP records a successful export.
Interrupted writes can leave unusable ciphertext, not plaintext. Password fields
clear after submission/cancel. Owned sensitive buffers are zeroized, but immutable
UI strings and third-party internals may have transient copies. This remains an
internal POC, not security-audited software.

## Older backups

The app still restores `.inheritance` format 1: `INHENC01`, 16-byte salt,
24-byte nonce and XChaCha20-Poly1305 ciphertext/tag, with the 48-byte header as
AAD. Its fixed Argon2id parameters are 19456 KiB, two iterations, one lane,
version 0x13 and a 32-byte key. Maximum legacy size is 131136 bytes.

New exports are ZIP only. Old plaintext files are not encrypted or removed.
Older app versions cannot restore these new ZIP backups.

## Device acceptance

Use a demo plan and a new test password, never real wallet secrets.

1. Reject short passwords and mismatched confirmation.
2. Export: exactly one new `.zip`, no readable sidecars.
3. Open with a desktop AES-ZIP tool; read the guide and inspect wallet.bsms.
4. Wrong password fails in both the desktop tool and Inheritance.
5. Restore the ZIP in Inheritance; preview first, then restore separately.
   Original plan unchanged; restored review/rehearsal/export assurances reset.
6. Restore an older encrypted `.inheritance` backup. Reject plaintext JSON/ZIP.
7. Cancel either prompt: no false export receipt. Edit plan: backup becomes stale.
8. Reopen the app: successful export status persists.
9. Disconnect the destination during export: no success or plaintext leak.
10. Check password keyboard Done, scrolling and responsiveness on hardware.
11. Import the decrypted wallet.bsms in compatible wallet software and compare the full policy,
    network and addresses. Do not send funds merely to test this POC.

Sources: [WinZip AES specification](https://www.winzip.com/en/support/aes-encryption/),
[7-Zip format support](https://www.7-zip.org/).
