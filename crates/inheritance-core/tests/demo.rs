use inheritance_core::import_kit;

#[test]
fn demo_signers_are_listed_on_separate_lines() {
    let plan = import_kit(include_bytes!("../../../tests/fixtures/demo-plan.json")).unwrap();
    for number in 1..=3 {
        assert!(plan.key_guidance.contains(&format!("\n• Signer {number}:")));
    }
    assert!(plan.key_guidance.ends_with("\n\nMatch their fingerprints."));
    assert_eq!(
        plan.guide().unwrap(),
        include_str!("../../../tests/fixtures/demo-guide.txt")
    );
}
