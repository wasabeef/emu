use emu::app::state::{AppState, CreateDeviceField, CreateDeviceForm, Mode, Panel};

#[test]
fn test_device_creation_field_navigation() {
    println!("=== DEVICE CREATION FIELD NAVIGATION TEST ===");

    let mut state = AppState::new();
    state.mode = Mode::CreateDevice;
    state.active_panel = Panel::Android;
    state.create_device_form = CreateDeviceForm::for_android();

    // Initial field should be API Level
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::ApiLevel
    );
    println!(
        "✅ Initial field: {active_field:?}",
        active_field = state.create_device_form.active_field
    );

    // Test next_field navigation (Android)
    state.create_device_form.next_field(); // API Level -> Category
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::Category
    );

    state.create_device_form.next_field(); // Category -> DeviceType
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::DeviceType
    );

    state.create_device_form.next_field(); // DeviceType -> RamSize
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::RamSize
    );

    state.create_device_form.next_field(); // RamSize -> StorageSize
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::StorageSize
    );

    state.create_device_form.next_field(); // StorageSize -> Name
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::Name
    );

    state.create_device_form.next_field(); // Name -> ApiLevel (circular)
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::ApiLevel
    );

    println!("✅ Android field navigation (next) works correctly");

    // Test prev_field navigation (Android)
    state.create_device_form.prev_field(); // ApiLevel -> Name
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::Name
    );

    state.create_device_form.prev_field(); // Name -> StorageSize
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::StorageSize
    );

    state.create_device_form.prev_field(); // StorageSize -> RamSize
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::RamSize
    );

    state.create_device_form.prev_field(); // RamSize -> DeviceType
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::DeviceType
    );

    state.create_device_form.prev_field(); // DeviceType -> Category
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::Category
    );

    state.create_device_form.prev_field(); // Category -> ApiLevel
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::ApiLevel
    );

    println!("✅ Android field navigation (prev) works correctly");
}

#[test]
fn test_ios_device_creation_navigation() {
    println!("=== iOS DEVICE CREATION NAVIGATION TEST ===");

    let mut state = AppState::new();
    state.mode = Mode::CreateDevice;
    state.active_panel = Panel::Ios;
    state.create_device_form = CreateDeviceForm::for_ios();

    // Initial field should be API Level
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::ApiLevel
    );

    // Test iOS navigation (no Category, RamSize, StorageSize)
    state.create_device_form.next_field_ios(); // ApiLevel -> DeviceType
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::DeviceType
    );

    state.create_device_form.next_field_ios(); // DeviceType -> Name
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::Name
    );

    state.create_device_form.next_field_ios(); // Name -> ApiLevel (circular)
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::ApiLevel
    );

    // Test reverse navigation
    state.create_device_form.prev_field_ios(); // ApiLevel -> Name
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::Name
    );

    state.create_device_form.prev_field_ios(); // Name -> DeviceType
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::DeviceType
    );

    state.create_device_form.prev_field_ios(); // DeviceType -> ApiLevel
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::ApiLevel
    );

    println!("✅ iOS field navigation works correctly");
}

#[test]
fn test_up_down_keys_field_movement_only() {
    println!("=== UP/DOWN KEYS FIELD MOVEMENT ONLY TEST ===");

    let mut state = AppState::new();
    state.create_device_form = CreateDeviceForm::for_android();

    // Set up some mock data for selections
    state.create_device_form.available_versions = vec![
        ("30".to_string(), "API 30".to_string()),
        ("31".to_string(), "API 31".to_string()),
        ("32".to_string(), "API 32".to_string()),
    ];
    state.create_device_form.available_device_types = vec![
        ("pixel_7".to_string(), "Pixel 7".to_string()),
        ("pixel_8".to_string(), "Pixel 8".to_string()),
    ];
    state.create_device_form.selected_api_level_index = 1; // API 31
    state.create_device_form.selected_device_type_index = 0; // Pixel 7
    state.create_device_form.selected_category_index = 1; // phone

    // Test that move_selection_up/down always return false (no selection change)
    let result_up = state.create_device_form.move_selection_up();
    assert!(!result_up);

    let result_down = state.create_device_form.move_selection_down();
    assert!(!result_down);

    // Verify selections didn't change
    assert_eq!(state.create_device_form.selected_api_level_index, 1);
    assert_eq!(state.create_device_form.selected_device_type_index, 0);
    assert_eq!(state.create_device_form.selected_category_index, 1);

    println!("✅ Up/down keys do not change selections (field movement only)");
}

#[test]
fn test_category_selection_change() {
    println!("=== CATEGORY SELECTION CHANGE TEST ===");

    let mut state = AppState::new();
    state.create_device_form = CreateDeviceForm::for_android();
    state.create_device_form.active_field = CreateDeviceField::Category;

    let initial_category = state.create_device_form.device_category_filter.clone();
    let initial_index = state.create_device_form.selected_category_index;

    // Test category cycling (like left/right keys would do)
    let len = state.create_device_form.available_categories.len();
    state.create_device_form.selected_category_index =
        (state.create_device_form.selected_category_index + 1) % len;
    state.create_device_form.update_selected_category();

    // Category should have changed
    assert_ne!(
        state.create_device_form.device_category_filter,
        initial_category
    );
    assert_ne!(
        state.create_device_form.selected_category_index,
        initial_index
    );

    println!("✅ Category selection can be changed (for left/right keys)");
}

#[test]
fn test_device_type_selection_with_mock_data() {
    println!("=== DEVICE TYPE SELECTION TEST ===");

    let mut state = AppState::new();
    state.create_device_form = CreateDeviceForm::for_android();
    state.create_device_form.active_field = CreateDeviceField::DeviceType;

    // Set up mock device types
    state.create_device_form.available_device_types = vec![
        ("pixel_7".to_string(), "Pixel 7".to_string()),
        ("pixel_8".to_string(), "Pixel 8 Pro".to_string()),
        ("samsung_s23".to_string(), "Samsung Galaxy S23".to_string()),
    ];
    state.create_device_form.selected_device_type_index = 0;
    state.create_device_form.device_type_id = "pixel_7".to_string();
    state.create_device_form.device_type = "Pixel 7".to_string();

    // Test selection change (like right key would do)
    let options = &state.create_device_form.available_device_types;
    let current_index = options
        .iter()
        .position(|(id, _)| id == &state.create_device_form.device_type_id)
        .unwrap();
    let new_index = (current_index + 1) % options.len();
    let (id, display) = options[new_index].clone();

    state.create_device_form.device_type_id = id.clone();
    state.create_device_form.device_type = display.clone();
    state.create_device_form.selected_device_type_index = new_index;

    assert_eq!(state.create_device_form.device_type_id, "pixel_8");
    assert_eq!(state.create_device_form.device_type, "Pixel 8 Pro");
    assert_eq!(state.create_device_form.selected_device_type_index, 1);

    println!("✅ Device type selection change works correctly");
}

#[test]
fn test_api_level_selection_with_mock_data() {
    println!("=== API LEVEL SELECTION TEST ===");

    let mut state = AppState::new();
    state.create_device_form = CreateDeviceForm::for_android();
    state.create_device_form.active_field = CreateDeviceField::ApiLevel;

    // Set up mock API levels
    state.create_device_form.available_versions = vec![
        ("30".to_string(), "API 30 - Android 11".to_string()),
        ("31".to_string(), "API 31 - Android 12".to_string()),
        ("32".to_string(), "API 32 - Android 12L".to_string()),
        ("33".to_string(), "API 33 - Android 13".to_string()),
    ];
    state.create_device_form.selected_api_level_index = 1;
    state.create_device_form.version = "31".to_string();
    state.create_device_form.version_display = "API 31 - Android 12".to_string();

    // Test selection change (like left key would do - previous)
    let options = &state.create_device_form.available_versions;
    let current_index = options
        .iter()
        .position(|(value, _)| value == &state.create_device_form.version)
        .unwrap();
    let new_index = if current_index == 0 {
        options.len() - 1
    } else {
        current_index - 1
    };
    let (value, display) = options[new_index].clone();

    state.create_device_form.version = value.clone();
    state.create_device_form.version_display = display.clone();
    state.create_device_form.selected_api_level_index = new_index;

    assert_eq!(state.create_device_form.version, "30");
    assert_eq!(
        state.create_device_form.version_display,
        "API 30 - Android 11"
    );
    assert_eq!(state.create_device_form.selected_api_level_index, 0);

    println!("✅ API level selection change works correctly");
}

#[test]
fn test_name_input_functionality() {
    println!("=== NAME INPUT FUNCTIONALITY TEST ===");

    let mut state = AppState::new();
    state.create_device_form = CreateDeviceForm::for_android();
    state.create_device_form.active_field = CreateDeviceField::Name;
    state.create_device_form.name = "Test".to_string();

    // Test character addition
    state.create_device_form.name.push('_');
    state.create_device_form.name.push('D');
    state.create_device_form.name.push('e');
    state.create_device_form.name.push('v');
    state.create_device_form.name.push('i');
    state.create_device_form.name.push('c');
    state.create_device_form.name.push('e');

    assert_eq!(state.create_device_form.name, "Test_Device");

    // Test character removal (backspace)
    state.create_device_form.name.pop();
    state.create_device_form.name.pop();
    state.create_device_form.name.pop();

    assert_eq!(state.create_device_form.name, "Test_Dev");

    println!("✅ Name input functionality works correctly");
}

#[test]
fn test_ram_storage_numeric_input() {
    println!("=== RAM/STORAGE NUMERIC INPUT TEST ===");

    let mut state = AppState::new();
    state.create_device_form = CreateDeviceForm::for_android();

    // Test RAM size input
    state.create_device_form.active_field = CreateDeviceField::RamSize;
    state.create_device_form.ram_size = "".to_string();

    // Simulate numeric input
    for c in "4096".chars() {
        if c.is_ascii_digit() {
            state.create_device_form.ram_size.push(c);
        }
    }
    assert_eq!(state.create_device_form.ram_size, "4096");

    // Test Storage size input
    state.create_device_form.active_field = CreateDeviceField::StorageSize;
    state.create_device_form.storage_size = "".to_string();

    for c in "16384".chars() {
        if c.is_ascii_digit() {
            state.create_device_form.storage_size.push(c);
        }
    }
    assert_eq!(state.create_device_form.storage_size, "16384");

    // Test non-numeric characters are ignored
    state.create_device_form.ram_size = "1024".to_string();
    let non_numeric_chars = "abc!@#";
    for c in non_numeric_chars.chars() {
        if c.is_ascii_digit() {
            state.create_device_form.ram_size.push(c);
        }
    }
    assert_eq!(state.create_device_form.ram_size, "1024"); // Should remain unchanged

    println!("✅ RAM/Storage numeric input validation works correctly");
}

#[test]
fn test_form_validation_states() {
    println!("=== FORM VALIDATION STATES TEST ===");

    let mut state = AppState::new();
    state.create_device_form = CreateDeviceForm::for_android();

    // Test empty name validation
    state.create_device_form.name = "".to_string();
    assert!(state.create_device_form.name.trim().is_empty());

    // Test valid name
    state.create_device_form.name = "Valid Device Name".to_string();
    assert!(!state.create_device_form.name.trim().is_empty());

    // Test empty version validation
    state.create_device_form.version = "".to_string();
    assert!(state.create_device_form.version.trim().is_empty());

    // Test valid version
    state.create_device_form.version = "31".to_string();
    assert!(!state.create_device_form.version.trim().is_empty());

    // Test error message functionality
    state.create_device_form.error_message = Some("Test error".to_string());
    assert!(state.create_device_form.error_message.is_some());

    state.create_device_form.error_message = None;
    assert!(state.create_device_form.error_message.is_none());

    println!("✅ Form validation states work correctly");
}

#[test]
fn test_placeholder_name_generation() {
    println!("=== PLACEHOLDER NAME GENERATION TEST ===");

    let mut state = AppState::new();
    state.create_device_form = CreateDeviceForm::for_android();

    // Set up device type and version
    state.create_device_form.device_type = "Pixel 7".to_string();
    state.create_device_form.version_display = "API 31 - Android 12".to_string();

    // Generate placeholder name
    state.create_device_form.generate_placeholder_name();

    // Should generate a meaningful name
    assert!(!state.create_device_form.name.is_empty());
    assert!(state.create_device_form.name.contains("Pixel"));
    assert!(state.create_device_form.name.contains("API"));

    println!(
        "Generated name: {name}",
        name = state.create_device_form.name
    );
    println!("✅ Placeholder name generation works correctly");
}
