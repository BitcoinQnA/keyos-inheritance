from pathlib import Path
import re
import unittest
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]


class UiContractTests(unittest.TestCase):
    def test_practice_reuses_full_guidance_key_details(self):
        app = (ROOT / "src/app.rs").read_text()
        source = (ROOT / "ui/app.slint").read_text()
        self.assertIn('1 => cards = signing_key_cards(p, &w)', app)
        self.assertIn('map(|w| signing_key_cards(p, &w))', app)
        practice = source.split('if View.page == 16: VerticalLayout', 1)[1].split('if View.page == 17:', 1)[0]
        self.assertIn('View.rehearsal-key-headings', practice)
        self.assertIn('View.rehearsal-key-details[index]', practice)
        self.assertNotIn('body: View.fingerprints', practice)
        helper = app.split('fn signing_key_cards(', 1)[1].split('fn guide(', 1)[0]
        for field in ['signer.name', 'signer.fingerprint', 'signer.location', 'signer.access', 'w.signer_role(index)', 'policy.normal_count', 'policy.threshold']:
            self.assertIn(field, helper)

    def test_copy_distinguishes_guidance_backup_and_optional_message(self):
        source = (ROOT / "ui/app.slint").read_text()
        app = (ROOT / "src/app.rs").read_text()
        self.assertIn('"Open Recovery Guide"', source)
        self.assertIn('"Export Encrypted Backup"', source)
        self.assertIn('"Save Practice Result"', source)
        for old in ['Signing key cards', 'Recover This Wallet', 'Export Recovery Kit']:
            self.assertNotIn(old, source)
        self.assertNotIn('open the separate kit', app)
        self.assertNotIn('missing(&p.message)', app)
        self.assertIn('if !p.message.trim().is_empty()', app)

    def test_checks_share_one_entry_and_status_rows_are_actionable(self):
        source = (ROOT / "ui/app.slint").read_text()
        detail = source.split("if View.page == 11: VerticalLayout", 1)[1].split("if View.page == 1:", 1)[0]
        self.assertIn('label: "Check Your Plan"', detail)
        self.assertNotIn("Actions.go(16)", detail)
        hub = source.split("if View.page == 17: VerticalLayout", 1)[1].split("VerticalLayout {", 1)[0]
        self.assertIn('PlanRow { heading: "Review Details"; detail: View.status', hub)
        self.assertIn('Actions.go(5)', hub)
        self.assertIn('Actions.go(16)', hub)
        self.assertIn('(optional)', hub)
        app = (ROOT / "src/app.rs").read_text()
        self.assertIn('(destination == 5 || destination == 16)', app)
        review = app.split('on!(on_review,', 1)[1].split('on!(on_next_step,', 1)[0]
        self.assertIn('rehearsed: plan.checklist.rehearsed', review)
        self.assertNotIn('rehearsed_at =', review)
        self.assertNotIn('You have rehearsed together (optional)', source)

    def test_additional_instructions_are_labelled_optional(self):
        source = (ROOT / "ui/app.slint").read_text()
        self.assertIn('heading: "Additional instructions (optional)"', source)
        self.assertNotIn('"How to get access"', source)
        self.assertNotIn('"Arrange access"', source)

    def setUp(self):
        self.source = (ROOT / "ui/app.slint").read_text()

    def test_menu_uses_sdk_rows_not_buttons(self):
        menu = self.source.split("if root.menu-open: Rectangle", 1)[1]
        self.assertIn("Menu {", menu)
        self.assertIn("MenuItem {", menu)
        self.assertNotIn("ActionButton", menu)
        self.assertIn("background: #0009", menu)

    def test_plan_overview_is_one_sdk_card_per_entry(self):
        self.assertIn("for name[index] in View.plan-names: Card", self.source)
        self.assertIn("Actions.select-plan(index)", self.source)
        overview = self.source.split("for name[index] in View.plan-names: Card", 1)[1].split("if View.page == 11", 1)[0]
        self.assertNotIn("icon:", overview)

    def test_guidance_uses_tokens_and_top_anchoring(self):
        self.assertIn("font-size: Theme.font-size-md", self.source)
        self.assertRegex(self.source, r"content := VerticalLayout\s*\{\s*x: 0px;\s*y: 0px;")
        self.assertIn("scroll.viewport-y = 0px", self.source)

    def test_guidance_uses_separate_cards(self):
        guide = self.source.split("if View.page == 6: VerticalLayout", 1)[1].split("if View.page == 7:", 1)[0]
        self.assertIn("in View.guide-headings: InfoCard", guide)
        self.assertIn("View.guide-details[index]", guide)
        self.assertNotIn("text: View.guide-body", guide)
        app = (ROOT / "src/app.rs").read_text()
        self.assertIn("for (index, signer) in care.signers.iter().enumerate()", app)
        self.assertIn('"When inheritance is available"', app)

    def test_rehearsal_confirmation_is_in_fixed_footer(self):
        body, footer = self.source.split('if View.page == 0: ActionButton', 1)
        self.assertNotIn('heading: View.rehearsal-confirmed', body)
        self.assertIn('"Confirm This Step Together"', footer)
        self.assertIn('"Skip Unfinished Step"', footer)
        self.assertIn('"Fix Missing Details"', footer)
        self.assertLess(footer.index('"Confirm This Step Together"'), footer.index('"Save Practice Result"'))

    def test_review_requires_fresh_non_persisted_confirmations(self):
        app = (ROOT / "src/app.rs").read_text()
        self.assertIn("state.review = [false; 3]", app)
        toggle = app.split("on!(on_check,", 1)[1].split("on!(on_review,", 1)[0]
        self.assertNotIn("commit(", toggle)
        self.assertIn("keys_located: state.review[0]", app)
        self.assertIn("disabled: !View.checks[0] || !View.checks[1] || !View.checks[2]", self.source)

    def test_sdk_icons_exist(self):
        icons = re.findall(r'Images\.icon\("([^\"]+)"', self.source)
        icons += ["user", "lock", "unlock", "download", "check"]
        directory = ROOT / ".foundation-sdk/current/lib/keyos/simulator/resources/icons"
        for icon in icons:
            self.assertTrue((directory / f"{icon}.svg").is_file(), icon)

    def test_guidance_icons_are_embedded_and_centered(self):
        guide = self.source.split("if View.page == 6: VerticalLayout", 1)[1].split("if View.page == 7:", 1)[0]
        self.assertIn("alignment: center", guide)
        self.assertIn("horizontal-alignment: center", guide)
        self.assertNotIn("Images.icon", guide)
        assets = re.findall(r'@image-url\("(icons/guide-[^\"]+)"\)', guide)
        self.assertEqual(len(assets), 5)
        for asset in assets:
            svg = ET.parse(ROOT / "ui" / asset).getroot()
            self.assertGreaterEqual(int(svg.attrib["width"]), 56)
            self.assertGreaterEqual(int(svg.attrib["height"]), 56)

    def test_primary_guidance_action_and_keyboard_exit(self):
        self.assertNotIn('if View.page == 4: BodyText { text: "Done"', self.source)
        self.assertIn('if View.page == 4: ActionButton { text: "Done"; clicked => { root.finish-edit(); } }', self.source)
        self.assertIn('if View.page == 11: ActionButton { text: "Heir Guidance";', self.source)
        self.assertNotIn('text: "Guide for My Heir"', self.source)
        self.assertIn("focus-reset.focus(); Actions.save-field();", self.source)

    def test_footer_primary_actions_follow_secondary_actions(self):
        footer = self.source.split('spacing: 12px;\n            if View.page == 0: ActionButton', 1)[1].split('if root.menu-open:', 1)[0]
        for secondary, primary in [
            ('text: "Scan Wallet QR"', 'text: "Choose Wallet File"'),
            ('"Create Separate Plan"', 'text: "Open Existing Plan"'),
            ('text: "Cancel"', '"Restore This Plan"'),
            ('"Restore Separate Plan"', 'text: "Open Existing Plan"'),
            ('"Save and Finish Later"', 'text: "Continue"'),
        ]:
            self.assertLess(footer.index(secondary), footer.index(primary))

    def test_keyboard_done_dismisses_before_multiline_input(self):
        editor = self.source.split('if View.page == 4: FocusScope', 1)[1].split('if View.page == 5:', 1)[0]
        self.assertIn('capture-key-pressed(event)', editor)
        self.assertIn('event.text == Key.Return', editor)
        self.assertIn('focus-reset.focus();', editor)
        self.assertIn('return accept;', editor)

    def test_backup_ui_and_export_are_encrypted_only(self):
        self.assertIn('input-type: password', self.source)
        self.assertIn('text: "Encrypt and Export"', self.source)
        self.assertIn('text: "Unlock Backup"', self.source)
        runtime = (ROOT / 'src/app.rs').read_text()
        writer = runtime.split('fn export_to(', 1)[1]
        self.assertIn('validate_encrypted_backup(bytes)?', writer)
        self.assertNotIn('plan.guide()', writer)
        self.assertNotIn('plan.bsms()', writer)
        self.assertNotIn('export_kit(', writer)
        self.assertIn('decrypt_backup(&bytes, &password)', runtime)
        self.assertIn('std::thread::Builder', runtime)

    def test_setup_signers_recovery_and_rehearsal_routes(self):
        for text in ['"Save and Finish Later"', '"Your Signing Keys"', '"Open Recovery Guide"', '"Save Practice Result"', '"Open Existing Plan"', '"Export Updated Backup"']:
            self.assertIn(text, self.source)
        self.assertIn("View.signer-fingerprint", self.source)
        self.assertIn("View.signer-fields[index]", self.source)
        self.assertIn("View.field-multiline", self.source)
        self.assertIn("not a lock or access restriction", self.source)


if __name__ == "__main__":
    unittest.main()
