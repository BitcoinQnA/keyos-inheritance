"""Independent AES ZIP checks using public demo data, entirely in memory."""
import io
import json
import subprocess
import pyzipper

exe = "target/release/examples/crypto_interop"
password = b"separate river lantern orchard"
backup = bytes.fromhex(subprocess.check_output([exe], text=True))
names = ["recovery-guide.txt", "wallet.bsms", "plan.json"]
with pyzipper.AESZipFile(io.BytesIO(backup)) as archive:
    assert archive.namelist() == names
    archive.setpassword(password)
    files = {name: archive.read(name) for name in names}
    assert json.loads(files["plan.json"])["plan"]["wallet_name"] == "Family Vault (demo)"
    assert b"Family Vault (demo)" in files["recovery-guide.txt"]
    assert files["wallet.bsms"].startswith(b"BSMS 1.0\n")
    for info in archive.infolist():
        assert info.flag_bits & 1 and info.wz_aes_strength == 3
    archive.setpassword(b"wrong password")
    try:
        archive.read(names[0])
    except RuntimeError:
        pass
    else:
        raise AssertionError("Wrong password accepted")

for name in names:
    plain = subprocess.check_output(
        ["/usr/bin/tar", "--passphrase", password.decode(), "-xOf", "-", name], input=backup)
    assert plain == files[name]

output = io.BytesIO()
with pyzipper.AESZipFile(output, "w", compression=pyzipper.ZIP_STORED,
                       encryption=pyzipper.WZ_AES) as archive:
    archive.setpassword(password)
    archive.setencryption(pyzipper.WZ_AES, nbits=256)
    for name in names:
        archive.writestr(name, files[name])
subprocess.run([exe, "decode"], input=output.getvalue(), check=True)

for problem in ["guide", "wallet", "extra", "plaintext", "compressed", "comment"]:
    changed = io.BytesIO()
    compression = pyzipper.ZIP_DEFLATED if problem == "compressed" else pyzipper.ZIP_STORED
    with pyzipper.AESZipFile(changed, "w", compression=compression, encryption=pyzipper.WZ_AES) as archive:
        archive.setpassword(password)
        archive.setencryption(pyzipper.WZ_AES, nbits=256)
        for name in names:
            data = files[name]
            if (problem == "guide" and name == names[0]) or (problem == "wallet" and name == names[1]):
                data += b"modified"
            if problem == "plaintext" and name == names[0]:
                archive.setencryption(None)
                archive.setpassword(None)
            archive.writestr(name, data)
            archive.setpassword(password)
            archive.setencryption(pyzipper.WZ_AES, nbits=256)
        if problem == "extra":
            archive.writestr("../unexpected.txt", b"unexpected")
        if problem == "comment":
            archive.comment = b"unsupported comment"
    result = subprocess.run([exe, "decode"], input=changed.getvalue(), capture_output=True)
    assert result.returncode != 0, problem
print("AES-256 ZIP: pyzipper both directions and macOS libarchive decryption passed")
