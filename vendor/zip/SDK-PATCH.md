# Foundation SDK random-source adaptation

Source: crates.io zip 2.4.2 (MIT), https://github.com/zip-rs/zip2.
Only runtime change: getrandom 0.3 `fill` becomes getrandom 0.2 `getrandom`.
The Foundation SDK's root Cargo patch routes 0.2 to the device TRNG.
No encryption algorithm, archive layout, or parser changes are made.
Only the `aes-crypto` feature is enabled; compression codecs are disabled.
