# Inheritance 0.3.0: device checks

Historical checklist. For v0.5.0 use [ENCRYPTED-BACKUP.md](ENCRYPTED-BACKUP.md).
Plaintext plan restore and plaintext export below are no longer supported.

Internal POC. Use the supplied public demo wallet only. Never fund it.

Install `Inheritance-0.3.0-beta3.app` through Settings > Apps on KeyOS
1.4.0-beta3 or later, with your existing QnA publisher certificate allowed.
This is an app installer, not a firmware image.

1. Restore `demo-plan.json` from this test kit using Plans > menu > Restore.
   Confirm a 2-of-3 test wallet, Alex Rivera, three separately named signer cards
   and wallet ID E03D375E. The restore must not claim a current export/rehearsal.
2. Open Edit Plan. Go through all five setup steps. Open each signer card;
   inspect its fingerprint, name, location and access instructions. Back out of
   an edited field before Done and verify the unsaved edit is discarded.
3. Clear one signer's access instructions, save, and inspect Plan Completeness.
   Two described keys are sufficient for this 2-of-3 plan. Clear another and
   expect a warning that only one has both required instructions. Restore the
   missing instructions before testing rehearsal. Do not enter any secrets.
4. Create a new plan from `demo-wallet.bsms`. Expect a duplicate-wallet warning.
   Open Existing Plan must not create another record. Repeat and choose Create
   Separate Plan: give it another name, Save and Finish Later, then resume it.
5. On the complete plan, open Heir Guidance > Recover This Wallet. Read all five
   steps. Scroll the longer signer list; Next must start the following step at
   its top. Icons and teal step headings should be centred and sharp.
6. Rehearse Together: Next without confirming must refuse. Confirm all three
   steps, scrolling when needed. Expect a user-confirmed rehearsal date in
   Plan Completeness. It is not proof of recovery and must not request a seed
   or a transaction.
7. Export a kit to the SD/USB drive. Expect three files with a shared prefix,
   a current revision receipt, a readable signer-by-signer guide and a BSMS file.
   Open the text on a computer. Check its name, revision/date and checklist.
8. Edit a signer name and save. Expect a newer revision, stale backup warning
   and reset rehearsal/review. Export Updated Kit must refresh the receipt.
   Cancelling the picker or removing a drive must not mark a failed export current.
9. Restore the exported JSON as a separate plan. Confirm it does not overwrite
   another plan or trust imported review/rehearsal/export receipts. Delete only
   that restored test copy; the other plans and exported files must remain.
10. Close/reopen the app, then restart the device. Confirm names, instructions,
    revisions and export status persist. Try the menu repeatedly, dismissing by
    tapping outside. Check light and dark themes and the software keyboard.

For an actual wallet, independently compare the imported public configuration
with Nunchuk and rehearse using the real signing devices. Physical touch,
removable-drive failures and cross-wallet recovery cannot be proven by simulator
tests. Keep a separate backup; do not rely solely on this internal POC.
