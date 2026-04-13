use super::*;

#[test]
fn test_extract_name_parts() {
    let config = DynamicDeviceConfig::new();

    let parts = config.extract_name_parts("Pixel 9 Pro (Google)");
    assert_eq!(parts, vec!["Pixel", "9", "Pro"]);

    let parts = config.extract_name_parts("Pixel 9 Pro Fold (Google)");
    assert_eq!(parts, vec!["Pixel", "9", "Pro", "Fold"]);

    let parts = config.extract_name_parts("Pixel 9 Pro XL (Google)");
    assert_eq!(parts, vec!["Pixel", "9", "Pro", "XL"]);

    let parts = config.extract_name_parts("Galaxy S23 Ultra (Samsung)");
    assert_eq!(parts, vec!["Galaxy", "S23", "Ultra"]);

    let parts = config.extract_name_parts("Nexus 5X");
    assert_eq!(parts, vec!["Nexus", "5X"]);

    let parts = config.extract_name_parts("Pixel Tablet (Google) (Large)");
    assert_eq!(parts, vec!["Pixel", "Tablet"]);
}

#[test]
fn test_parse_device_name() {
    let config = DynamicDeviceConfig::new();

    let parts = config.parse_device_name("Pixel 9 Pro Fold (Google)");
    assert_eq!(parts, vec!["Pixel", "9", "Pro", "Fold"]);

    let parts = config.parse_device_name("Pixel 9 Pro XL (Google)");
    assert_eq!(parts, vec!["Pixel", "9", "Pro", "XL"]);
}

#[test]
fn test_placeholder_generation() {
    use crate::app::state::CreateDeviceForm;

    let mut form = CreateDeviceForm::new();
    form.device_type = "Pixel 9 Pro Fold (Google)".to_string();
    form.version_display = "API 36 - Android 15".to_string();

    form.generate_placeholder_name();
    assert_eq!(form.name, "Pixel 9 Pro Fold API 36");

    form.device_type = "Pixel 9 Pro XL (Google)".to_string();
    form.generate_placeholder_name();
    assert_eq!(form.name, "Pixel 9 Pro XL API 36");

    form.device_type = "Pixel 9 Pro (Google)".to_string();
    form.generate_placeholder_name();
    assert_eq!(form.name, "Pixel 9 Pro API 36");
}
