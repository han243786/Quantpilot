use super::*;
use crate::parse_quant_script_module;

mod basic_runtime_smoke_tests;
mod manual_formula_tests;
mod rebalance_lowering_tests;
mod spread_lowering_tests;

#[test]
fn rejects_semantic_errors_before_runtime_lowering() {
    let module = parse_quant_script_module(
        r#"
fn strategy() {
    let signal = missing_helper(1)
    if 42 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
    if signal > 0 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
    )
    .unwrap();

    let err = lower_script_to_runtime_config(&module).unwrap_err();
    let message = err.to_string();
    assert!(message.contains("QS0005"));
    assert!(message.contains("missing_helper"));
    assert!(message.contains("QS0006"));
}
