use inheritance_core::{export_kit, import_kit};

fn main() {
    let output = std::env::args().nth(1).expect("output folder required");
    let mut plan = import_kit(include_bytes!("../../../tests/fixtures/demo-plan.json")).unwrap();
    plan.upgrade().unwrap();
    plan.key_guidance = "Demo only. Match each fingerprint. Never fund this test wallet.".into();
    for (index, signer) in plan.care.as_mut().unwrap().signers.iter_mut().enumerate() {
        signer.name = ["Home Passport", "Executor backup", "Alex's device"][index].into();
        signer.location = ["Home safe", "Executor office", "Alex's safe"][index].into();
        signer.access =
            "Ask Sam for the separate access instructions. No PINs or passwords are stored here."
                .into();
    }
    plan.revise(None, 1788849000).unwrap();
    std::fs::create_dir_all(&output).unwrap();
    let output = std::path::Path::new(&output);
    std::fs::write(output.join("demo-plan.json"), export_kit(&plan).unwrap()).unwrap();
    std::fs::write(output.join("demo-guide.txt"), plan.guide().unwrap()).unwrap();
    std::fs::write(output.join("demo-wallet.bsms"), plan.bsms().unwrap()).unwrap();
}
