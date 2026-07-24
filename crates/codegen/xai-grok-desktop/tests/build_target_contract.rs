#[test]
fn script_supports_all_product_targets() {
    let script = include_str!("../../../../scripts/build-windows.ps1");

    assert!(script.contains("ValidateSet('terminal', 'desktop', 'both')"));
}
