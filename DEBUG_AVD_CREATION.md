# Android AVD Creation Debug Guide

This document outlines the debugging improvements made to diagnose "Failed to create Android AVD" errors.

## Debug Tools Added

### 1. Enhanced CommandRunner Logging

The `CommandRunner` now provides detailed debug output for all command executions:

```rust
// Shows the exact command being executed
[DEBUG] Executing command: /path/to/avdmanager create avd -n test_device -k system-images;android-34;google_apis_playstore;arm64-v8a --force
[DEBUG] Command exit code: Some(1)
[DEBUG] Command stdout: 
[DEBUG] Command stderr: Error: Package path is not valid. Valid system image paths are:
```

### 2. Enhanced Android Manager

Added comprehensive debugging to the Android AVD creation process:

- **SDK Path Detection**: Shows the detected Android SDK home directory and tool paths
- **System Image Validation**: Checks if the required system image is installed before attempting creation
- **Device Name Validation**: Validates device names against Android's naming requirements
- **Better Error Messages**: Provides specific error messages for common issues

### 3. Debug Binary

Created `debug-avd` binary that can be run independently to diagnose issues:

```bash
cargo run --bin debug-avd
```

This tool:
- Tests Android Manager initialization
- Lists available system images
- Lists existing AVDs
- Attempts to create a test AVD
- Provides cleanup and diagnostic information

## Common Issues and Solutions

### 1. System Image Not Found

**Error**: `System image 'system-images;android-XX;google_apis_playstore;arm64-v8a' not found`

**Solution**: Install the required system image:
```bash
sdkmanager "system-images;android-34;google_apis_playstore;arm64-v8a"
```

### 2. License Not Accepted

**Error**: Contains "license" in stderr

**Solution**: Accept Android SDK licenses:
```bash
sdkmanager --licenses
```

### 3. Invalid Device Name

**Error**: Device name contains invalid characters

**Solution**: The device name sanitization function (`sanitize_device_name`) converts invalid characters to underscores, but some names might still be rejected by avdmanager.

### 4. SDK Tools Not Found

**Error**: `Tool 'avdmanager' not found in Android SDK`

**Solution**: 
- Verify ANDROID_HOME or ANDROID_SDK_ROOT environment variable is set
- Ensure Android SDK command-line tools are installed
- Check that the tools are in the expected directories

## Debugging Process

### Step 1: Run the Debug Tool

```bash
cargo run --bin debug-avd
```

This will show:
- Whether Android Manager can be initialized
- What system images are available
- Whether test AVD creation works

### Step 2: Check Command Output

The enhanced logging will show the exact commands being executed and their outputs. Look for:

1. **Command being executed**: Is the avdmanager command correct?
2. **Exit code**: Non-zero indicates failure
3. **stderr content**: Contains the actual error message
4. **System image availability**: Is the required system image installed?

### Step 3: Verify Prerequisites

Ensure these are properly set up:

1. **Android SDK**: Properly installed and ANDROID_HOME set
2. **System Images**: Required system images are installed
3. **Licenses**: Android SDK licenses are accepted
4. **Tools**: avdmanager and sdkmanager are available

### Step 4: Test with Different Parameters

Try different combinations:
- Different API levels (34, 33, 32, etc.)
- Different ABIs (arm64-v8a, x86_64)
- Different tags (google_apis, google_apis_playstore, default)

## Code Changes Made

### CommandRunner (`src/utils/command.rs`)
- Added debug logging for all command executions
- Shows command, exit code, stdout, and stderr

### AndroidManager (`src/managers/android.rs`)
- Added `check_system_image_available()` method
- Added `list_available_system_images()` method
- Enhanced device creation with pre-validation
- Better error messages for common issues
- Device name validation

### Debug Binary (`src/bin/debug_avd.rs`)
- Standalone tool for testing AVD creation
- Step-by-step diagnostic process
- Automatic cleanup of test devices

## Usage Example

```bash
# Build the debug tool
cargo build --bin debug-avd

# Run the debug tool
cargo run --bin debug-avd

# Or run the main application with debug output
RUST_LOG=debug cargo run

# Create a device with the enhanced debugging
# (The debug output will show exactly what's happening)
```

## Environment Variables for Debugging

Set these for additional debugging:
```bash
export RUST_LOG=debug
export ANDROID_VERBOSE=1
```

This debugging enhancement should help identify the exact cause of AVD creation failures and provide actionable solutions.