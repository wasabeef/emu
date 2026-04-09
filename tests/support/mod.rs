//! Unified test infrastructure for integration tests.
//!
//! All shared test utilities live here — device factories, state builders,
//! mock managers, fixture loaders, custom assertions, and trait contract tests.

pub mod assertions;
pub mod contract;
pub mod devices;
pub mod fixtures;
pub mod managers;
pub mod state;

// Re-export commonly used items for convenience
pub use devices::{android_device, android_device_with_status, ios_device, ios_device_with_status};
pub use state::TestStateBuilder;
