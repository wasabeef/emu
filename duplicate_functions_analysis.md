# Duplicate and Similar Functions Analysis for Emu Codebase

## 1. Device Start/Stop Operations

### Android Manager (`src/managers/android.rs`)
```rust
async fn start_device(&self, identifier: &str) -> Result<()> {
    let args = vec![
        "-avd", identifier,
        "-no-audio",
        "-no-snapshot-save",
        "-no-boot-anim",
        "-netfast",
    ];
    self.command_runner.spawn(&self.emulator_path, &args).await?;
    Ok(())
}

async fn stop_device(&self, identifier: &str) -> Result<()> {
    let running_avds = self.get_running_avd_names().await?;
    if let Some(emulator_id) = running_avds.get(identifier) {
        self.command_runner
            .run("adb", &["-s", emulator_id, "emu", "kill"])
            .await
            .context(format!("Failed to stop emulator {}", emulator_id))?;
    }
    Ok(())
}
```

### iOS Manager (`src/managers/ios.rs`)
```rust
async fn start_device(&self, identifier: &str) -> Result<()> {
    // Check if already booted (duplicated logic)
    let status_output = self.command_runner
        .run("xcrun", &["simctl", "list", "devices", "-j"])
        .await
        .context("Failed to get device status")?;
    
    // Parse JSON and check state (duplicated pattern)
    let json: Value = serde_json::from_str(&status_output)
        .context("Failed to parse device status")?;
    
    // ... checking if already booted ...
    
    // Boot device
    let boot_result = self.command_runner
        .run("xcrun", &["simctl", "boot", identifier])
        .await;
    
    // Error handling pattern duplicated
    match boot_result {
        Ok(_) => log::info!("Successfully booted iOS device {}", identifier),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("Unable to boot device in current state: Booted") {
                log::info!("Device {} was already in the process of booting", identifier);
            } else {
                return Err(e).context(format!("Failed to boot iOS device {}", identifier));
            }
        }
    }
    
    // Open Simulator app
    if let Err(e) = self.command_runner.spawn("open", &["-a", "Simulator"]).await {
        log::warn!("Failed to open Simulator app: {}", e);
    }
    Ok(())
}

async fn stop_device(&self, identifier: &str) -> Result<()> {
    let shutdown_result = self.command_runner
        .run("xcrun", &["simctl", "shutdown", identifier])
        .await;
    
    // Similar error handling pattern
    match shutdown_result {
        Ok(_) => {
            log::info!("Successfully shut down iOS device {}", identifier);
            Ok(())
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("Unable to shutdown device in current state: Shutdown") {
                log::info!("Device {} was already shut down", identifier);
                Ok(())
            } else {
                Err(e).context(format!("Failed to shutdown iOS device {}", identifier))
            }
        }
    }
}
```

**Refactoring Opportunities:**
- Extract common device state checking logic
- Create a generic error handler for "already in state" errors
- Unify logging patterns

## 2. Error Handling Patterns

### Common Pattern: Context with Formatting
```rust
// Found throughout the codebase:
.context(format!("Failed to {} {}", action, identifier))?
.context(format!("Failed to {} iOS device {}", action, identifier))
.context(format!("Failed to {} Android device {}", action, identifier))
```

### Common Pattern: State Error Handling
```rust
// iOS Manager pattern:
if error_msg.contains("Unable to boot device in current state: Booted") {
    log::info!("Device {} was already in the process of booting", identifier);
} else {
    return Err(e).context(...);
}

// Android Manager doesn't have this graceful handling
```

**Refactoring Opportunities:**
- Create a `DeviceError` enum with common error types
- Extract error message parsing into utility functions
- Standardize context message formatting

## 3. Command Execution Patterns

### CommandRunner Usage Patterns
```rust
// Pattern 1: Run and wait
self.command_runner.run(program, args).await.context(...)?

// Pattern 2: Spawn background
self.command_runner.spawn(program, args).await?

// Pattern 3: Run with error tolerance
let _ = self.command_runner.run(...).await; // Ignore errors
```

**Refactoring Opportunities:**
- Add convenience methods like `run_ignoring_errors`
- Create builder pattern for complex commands
- Add retry logic for transient failures

## 4. UI Rendering Patterns for Dialogs

### Confirmation Dialog Pattern (Duplicated)
```rust
fn render_confirm_delete_dialog(...) {
    // Calculate dimensions
    let dialog_width = 50.min(size.width - 4);
    let dialog_height = 8.min(size.height - 4);
    
    // Center dialog
    let x = (size.width.saturating_sub(dialog_width)) / 2;
    let y = (size.height.saturating_sub(dialog_height)) / 2;
    
    // Clear and render background
    frame.render_widget(Clear, dialog_area);
    frame.render_widget(background_block, dialog_area);
    
    // Render dialog with title, message, and shortcuts
}

fn render_confirm_wipe_dialog(...) {
    // EXACT SAME PATTERN as above, only title and message differ
}
```

**Refactoring Opportunities:**
- Extract generic `render_confirmation_dialog` function
- Create `DialogConfig` struct with title, message, style
- Reuse layout calculations

### Form Field Rendering Pattern
```rust
render_input_field(...);
render_select_field(...);
// Repeated for each field with similar parameters
```

**Refactoring Opportunities:**
- Create a `FormRenderer` struct
- Use a loop with field definitions
- Extract common styling logic

## 5. Form Validation Logic

### Implicit Validation Patterns
```rust
// Name validation (scattered):
let safe_name = config.name.chars()
    .filter_map(|c| match c {
        c if c.is_ascii_alphanumeric() || c == '.' || c == '-' => Some(c),
        ' ' | '_' => Some('_'),
        _ => None,
    })
    .collect::<String>()
    .trim_matches('_')
    .to_string();

if safe_name.is_empty() {
    return Err(anyhow::anyhow!("Device name '{}' contains only invalid characters"));
}

// RAM/Storage validation (implicit):
// No validation found - accepting any string
```

**Refactoring Opportunities:**
- Create `ValidationRule` trait
- Implement common validators (numeric range, name format, etc.)
- Centralize validation logic

## 6. Device Name Sanitization (Multiple Implementations)

```rust
// In common.rs:
pub fn sanitize_device_name(name: &str) -> String { ... }
pub fn sanitize_device_name_for_command(name: &str) -> String { ... }

// In android.rs create_device:
let safe_name = config.name.chars()
    .filter_map(|c| match c { ... })
    .collect::<String>();
```

**Refactoring Opportunities:**
- Use existing sanitization functions consistently
- Create platform-specific sanitizers if needed
- Document sanitization rules clearly

## 7. JSON Parsing Patterns

```rust
// Repeated pattern:
let output = self.command_runner.run(...).await?;
let json: Value = serde_json::from_str(&output)
    .context("Failed to parse JSON")?;

// Device extraction pattern:
if let Some(devices) = json.get("devices").and_then(|v| v.as_object()) {
    for (_, device_list) in devices {
        if let Some(devices_array) = device_list.as_array() {
            // Process devices
        }
    }
}
```

**Refactoring Opportunities:**
- Create typed structs for JSON responses
- Extract JSON navigation helpers
- Add error recovery for malformed JSON

## 8. Background Task Patterns

```rust
// Loading pattern (repeated):
tokio::spawn(async move {
    // Load data
    // Update shared state
});

// Progress update pattern:
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(2)).await;
        // Update progress
    }
});
```

**Refactoring Opportunities:**
- Create `BackgroundTask` abstraction
- Implement progress reporter trait
- Add task cancellation support

## Recommended Refactorings

### Priority 1: Extract Common Dialog Renderer
```rust
pub fn render_confirmation_dialog(
    frame: &mut Frame,
    title: &str,
    message: &str,
    icon: &str,
    border_color: Color,
    theme: &Theme,
) {
    // Common dialog rendering logic
}
```

### Priority 2: Device Operation Traits
```rust
pub trait DeviceOperations {
    async fn check_device_state(&self, identifier: &str) -> Result<DeviceState>;
    async fn handle_state_error(&self, error: anyhow::Error, expected_state: &str) -> Result<()>;
}
```

### Priority 3: Form Validation Framework
```rust
pub trait FieldValidator {
    fn validate(&self, value: &str) -> Result<(), String>;
}

pub struct NumericRangeValidator { min: u32, max: u32 }
pub struct DeviceNameValidator { platform: Platform }
```

### Priority 4: Command Execution Helpers
```rust
impl CommandRunner {
    pub async fn run_ignoring_state_errors(&self, program: &str, args: &[&str]) -> Result<()>;
    pub async fn run_with_retry(&self, program: &str, args: &[&str], retries: u32) -> Result<String>;
}
```

### Priority 5: Error Type Unification
```rust
#[derive(Debug, thiserror::Error)]
pub enum DeviceOperationError {
    #[error("Device already in state: {0}")]
    AlreadyInState(String),
    
    #[error("Command failed: {0}")]
    CommandFailed(String),
    
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
}
```

These refactorings would significantly reduce code duplication, improve maintainability, and make the codebase more consistent across platforms.