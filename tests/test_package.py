import copy
import importlib.util
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("pack", ROOT / "scripts/pack-beta3.py")
pack = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pack)


class PackageTests(unittest.TestCase):
    def setUp(self):
        self.config = pack.read_config(ROOT)
        self.files = pack.read_bundle(ROOT / f"target/Inheritance-{self.config['version']}-beta3.app")

    def change_manifest(self, change):
        manifest = json.loads(pack.signed_payload(self.files["manifest.json"]))
        change(manifest)
        self.files["manifest.json"] = self.files["manifest.json"][:2048] + json.dumps(manifest).encode()

    def test_valid_release(self):
        pack.validate(self.files, self.config)

    def test_changed_binary_rejected(self):
        self.files["app.elf"] += b"changed"
        with self.assertRaises(ValueError):
            pack.validate(self.files, self.config)

    def test_missing_minimum_rejected(self):
        self.change_manifest(lambda m: m.pop("minKeyosVersion"))
        with self.assertRaises(ValueError):
            pack.validate(self.files, self.config)

    def test_wrong_app_rejected(self):
        self.change_manifest(lambda m: m.update(appId="0x1234"))
        with self.assertRaises(ValueError):
            pack.validate(self.files, self.config)

    def test_extra_file_rejected(self):
        self.files["extra.txt"] = b"extra"
        with self.assertRaises(ValueError):
            pack.validate(self.files, self.config)

    def test_missing_signature_rejected(self):
        self.files["app.elf"] = b"\0" * 2048 + self.files["app.elf"][2048:]
        with self.assertRaises(ValueError):
            pack.validate(self.files, self.config)

    def test_no_security_or_network_permissions(self):
        import tomllib
        config = tomllib.loads((ROOT / "app-config.toml").read_text())
        templates = tomllib.loads((ROOT / "permission_templates.toml").read_text())
        permissions = copy.deepcopy(config["permissions"])
        for template in permissions.pop("template"):
            permissions.update(templates[template])
        self.assertEqual(set(permissions), {"os/fs", "os/gui-server", "os/settings"})


if __name__ == "__main__":
    unittest.main()
