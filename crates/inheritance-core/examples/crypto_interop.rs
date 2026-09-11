#[path = "../tests/support/mod.rs"]
mod support;
use inheritance_core::{decrypt_backup, encrypt_backup};
use std::io::Read;
fn main() {
    let password = "separate river lantern orchard";
    if std::env::args().nth(1).as_deref() == Some("decode") {
        let mut bytes = Vec::new();
        std::io::stdin().read_to_end(&mut bytes).unwrap();
        assert_eq!(decrypt_backup(&bytes, password).unwrap(), support::plan());
        println!("Independent encrypted backup restored successfully");
    } else {
        for b in encrypt_backup(&support::plan(), password).unwrap() {
            print!("{b:02x}");
        }
    }
}
