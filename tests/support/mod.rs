//! Unified test infrastructure for integration tests.
//!
//! All shared test utilities live here — device factories, state builders,
//! mock managers, fixture loaders, custom assertions, and trait contract tests.
//!
//! Not all items are used by every test binary. Suppress dead_code warnings
//! since this module serves as shared infrastructure across multiple binaries.

#[allow(dead_code)]
pub mod assertions;
#[allow(dead_code)]
pub mod contract;
#[allow(dead_code)]
pub mod devices;
#[allow(dead_code)]
pub mod fixtures;
#[allow(dead_code)]
pub mod managers;
#[allow(dead_code)]
pub mod state;

// Re-export commonly used items for convenience
pub use devices::{android_device, android_device_with_status, ios_device, ios_device_with_status};
pub use state::TestStateBuilder;
