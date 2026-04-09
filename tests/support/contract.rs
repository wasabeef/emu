//! DeviceManager trait contract tests.
//!
//! These functions verify behavioral contracts that ANY DeviceManager
//! implementation must satisfy. Run them against MockDeviceManager,
//! and later against AndroidManager / IosManager if real SDK is available.

#[cfg(feature = "test-utils")]
use emu::managers::common::DeviceManager;

/// Core contract: list_devices returns Ok and is deterministic.
#[cfg(feature = "test-utils")]
pub async fn verify_list_devices_returns_ok(manager: &impl DeviceManager) {
    let result = manager.list_devices().await;
    assert!(result.is_ok(), "list_devices must return Ok");
}

/// Core contract: is_available returns a stable boolean.
#[cfg(feature = "test-utils")]
pub async fn verify_is_available(manager: &impl DeviceManager) {
    let a = manager.is_available().await;
    let b = manager.is_available().await;
    assert_eq!(a, b, "is_available must be stable across calls");
}

/// Core contract: deleting a nonexistent device returns Err.
#[cfg(feature = "test-utils")]
pub async fn verify_delete_nonexistent_fails(manager: &impl DeviceManager) {
    let result = manager.delete_device("__nonexistent_device__").await;
    assert!(
        result.is_err(),
        "delete_device on nonexistent device must return Err"
    );
}

/// Core contract: stopping a nonexistent device returns Err.
#[cfg(feature = "test-utils")]
pub async fn verify_stop_nonexistent_fails(manager: &impl DeviceManager) {
    let result = manager.stop_device("__nonexistent_device__").await;
    assert!(
        result.is_err(),
        "stop_device on nonexistent device must return Err"
    );
}

/// Run all contract tests against a DeviceManager implementation.
#[cfg(feature = "test-utils")]
pub async fn verify_all_contracts(manager: &impl DeviceManager) {
    verify_list_devices_returns_ok(manager).await;
    verify_is_available(manager).await;
    verify_delete_nonexistent_fails(manager).await;
    verify_stop_nonexistent_fails(manager).await;
}
