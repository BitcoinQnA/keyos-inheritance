#[path = "../tests/support/mod.rs"]
mod support;
fn main() {
    let dir = std::env::args().nth(1).expect("output directory");
    std::fs::create_dir_all(&dir).unwrap();
    let plan = support::plan();
    std::fs::write(format!("{dir}/demo-wallet.bsms"), plan.bsms().unwrap()).unwrap();
    std::fs::write(
        format!("{dir}/demo-plan.json"),
        inheritance_core::export_kit(&plan).unwrap(),
    )
    .unwrap();
    std::fs::write(format!("{dir}/demo-guide.txt"), plan.guide().unwrap()).unwrap();
}
