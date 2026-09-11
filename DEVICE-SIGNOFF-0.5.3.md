# Inheritance 0.5.3: device sign-off

Use test data only. Do not move bitcoin or enter seed words, device PINs or real passwords into plan notes. Use a separate test password for encrypted exports.

## 1. Install and existing plans

- Install `Inheritance-0.5.3-beta3.app`. About should show 0.5.3 and the internal-POC/master-seed limitations.
- Existing plans, signer names and notes should remain unchanged after updating.
- Open and dismiss the three-dot menu ten times. It should respond each time, dim the background and close when tapping outside.

## 2. Fresh 2-of-3 onboarding

- Create a Plan and choose `TEST-ONLY-2of3-Onboarding-20260909.bsms` from the SD card. Expect testnet, 2 of 3 and no timelock.
- Compare the fingerprints and first address with the file. If already imported, choose Create Separate Plan to start fresh; the existing plan must remain untouched.
- Name the plan. Under Your Signing Keys, give two different keys a name, location and access instructions. A device and its seed backup count as the same key.
- Add an heir and trusted contact. Leave Personal Message, Additional Key Notes and Additional Instructions blank. These are optional.
- Change the review interval. It should explain that this is an in-app reminder, not a notification.
- Save and Finish Later halfway through, reopen and continue. Save the completed plan, close/reopen the app and confirm the details persist.
- In text fields, keyboard Done should dismiss the keyboard; the bottom Done should save. There should be no duplicate top-right Done.

## 3. Missing details and review

- On a separate incomplete test plan, open Check Your Plan and tap Review Details / Not reviewed yet. Expect a missing-details explanation and a working Fix Missing Details button, not a silent no-op.
- Add the missing details. Review Details should now open three fresh confirmations. The Record button must stay disabled until all three are checked.
- Check one item and go back without recording. The previous review date must remain unchanged; reopening requires fresh checks.
- Export a test backup, then complete and record the review. Expect a Last reviewed date and the next review interval.
- Practice must still say Not practised yet. A review must not claim practice has occurred.

## 4. Practice with your heir

- From Check Your Plan, choose Practise with Your Heir. Confirm each of the three steps; Next / Record Practice must remain disabled until the current step is confirmed.
- Undo a confirmation, go back and forward, and scroll the final wallet-details card fully. The confirmation and primary buttons must remain reachable.
- Record practice. Expect a separate Last practised date. Record another review and confirm the practice date is preserved.
- Editing plan instructions should invalidate previous assurances and make an existing backup out of date.

## 5. Heir guidance and inheritance policies

- Open Heir Guidance, then Open Recovery Guide. Follow all five pages, forwards and backwards. Empty optional messages must not appear as missing required information.
- Check key names, roles, fingerprints, locations and access instructions on their separate cards. Scroll long content to the end.
- Import your Nunchuk inheritance BSMS and descriptor exports as well. Compare normal/inheritance thresholds, key roles, timelock and first address with Nunchuk. Normal and inheritance keys must not be counted together.
- Confirm the app says Nunchuk must check blockchain eligibility. A displayed date must not imply that this app can unlock funds.

## 6. Encrypted backup and restore

- Export Encrypted Backup. Try mismatched passwords and a password under 20 characters; both must be rejected. Cancel the destination picker; there should be no success claim.
- Use a test password of at least 20 characters. Export to the SD card. Expect one encrypted ZIP and a success message, not plaintext companion files.
- On a computer, use an AES-256 ZIP-compatible application to open it. Check `recovery-guide.txt`, `wallet.bsms` and `plan.json` are present and correspond to the selected plan. Extract only to a private test folder; extracted contents are plaintext.
- Restore Plan in the app: a wrong password must fail without adding a plan. The right password should show a preview before saving.
- Restore as a separate plan. Original plans must remain unchanged, and imported review/practice/export assurances must reset.
- Import the decrypted `wallet.bsms` into Nunchuk and compare the network and first receive address. Do not send funds. The encrypted ZIP itself is not a Nunchuk wallet file.
- Try a damaged ZIP and an unrelated text file. Expect a clear error and no new or overwritten plan.

## 7. Multiple plans, deletion and layout

- Edit one of two plans; the other must not change. Duplicate-wallet import should offer the existing plan and a separate-plan option.
- Cancel deletion first, then delete only the disposable duplicate. Other plans and exported files must remain.
- Check light and dark themes, long names/notes and all scrolling pages. No clipped text, inaccessible buttons or unexplained blank areas. Teal primary actions should be below secondary actions in the footer.

Record any failure with the section number, exact taps, screenshot and version shown in About. This is an internal POC; passing this checklist is not a guarantee of real-world recovery.
