use rust_value_betting_engine::crate_name;

#[test]
fn exposes_crate_name() {
    assert_eq!(crate_name(), "rust-value-betting-engine");
}
