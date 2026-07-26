#[test]
fn has_the_expected_desktop_identity() {
    assert_eq!(xai_grok_desktop::PRODUCT_NAME, "Grok Desktop");
    assert_eq!(xai_grok_desktop::BINARY_NAME, "grok-desktop");
}
