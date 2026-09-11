# Inheritance 0.6.0: safety and usability sign-off

Internal POC. Test data only. Do not move funds. Automated and simulated checks do not prove that a family can recover a wallet.

## What changed

- Recovery companion positioning; information entered is not recovery readiness.
- Minimal custodian/location references, with unlocking credentials kept separate.
- Handover arrangements record four confirmations, never passwords or PINs.
- Five-step practice stores unfinished results honestly and separately from review.
- Full signing-key details appear within practice. The home-screen short ID is removed.
- Exported instructions distinguish heir recovery from owner preparation and explain privacy risks.
- Older plans and encrypted backups can be read. New fields require this version; export a backup before any downgrade. Old software may reject newly saved plans.

## Owner device checks

1. Install 0.6.0, check About, and confirm existing plans remain. Use a disposable separate plan for tests below.
2. Import the 2-of-3 test BSMS. Use a custodian reference rather than an exact key location. Enter no PINs, passwords or seeds. Leave optional notes blank and complete setup.
3. Open Check Your Plan. Information entered must explicitly not guarantee recovery. Tap Not reviewed yet; confirm the review opens or identifies missing details.
4. Open Handover Arrangements. Save only one confirmation. Reopen and confirm 1 of 4 remains; other arrangements must not be implied. Arrange the real access route outside this locked plan.
5. Practise with Your Heir: skip at least one step, confirm others and save. The result must show the correct number out of five, not a success badge. Reopen: new practice must start with fresh confirmations.
6. During the key step, scroll through names, roles, fingerprints, custodian/location and access references. With an inheritance wallet, compare both path thresholds and timelock with the source wallet software.
7. Finish all five practice steps using test data and save. Then record a details review. The practice record must remain separate. Editing instructions must reset confirmations and make the backup stale.
8. Export with matching test passwords of at least 20 characters. Wrong/mismatched passwords and picker cancellation must not claim success. Expect only an encrypted ZIP.
9. Open the ZIP independently with an AES-256 ZIP tool. Read the guide and import wallet.bsms into compatible wallet software. Compare the policy, network and first receive address; do not send funds. Extracted files are plaintext: use a private test folder.
10. Restore both a pre-0.6.0 encrypted backup and a new one as separate plans. Confirm names/notes survive, originals remain, and handover/review/practice assurances reset. Wrong passwords and damaged files must add nothing.
11. Exercise back/cancel, keyboard Done, long text, both themes and repeated menu opening. No inaccessible buttons; one simulator maximum if using a simulator.

## Uncoached outsider test: required before promoting beyond POC

Use a volunteer unfamiliar with the app and a fictional wallet. Give them the device or encrypted backup and an independently accessible test-password arrangement. Do not use real custody details.

Ask them, without coaching, to:

1. Explain what the app can and cannot do.
2. Find the plan, identify the intended heir and someone to contact.
3. Explain how many distinct keys are required and whom to approach for them.
4. Open the backup on a computer and find the guide and wallet configuration.
5. Import the wallet configuration into compatible wallet software and compare its policy, network and first address.
6. Describe what to do if the device, one copy or first contact is unavailable.
7. Identify information that must never be stored in the plan.

Record every hesitation, wrong assumption and request for help. A coached completion is not a pass. Stop and fix the journey where they get stuck. Passing does not demonstrate possession of real signing keys or guarantee recovery.

## Remaining sign-off gates

Physical-device acceptance, an actual current wallet-software import comparison and the uncoached outsider test must be performed by the testers. Do not describe them as passed based on software tests alone. No automated check can prove that a separate password arrangement or alternative custodian is genuinely available.
