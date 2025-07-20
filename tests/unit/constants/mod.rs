//! Unit tests for the constants module
//!
//! This module provides comprehensive test coverage for all constants modules,
//! focusing on executable code, runtime behavior, and constant validation.
//!
//! # Test Coverage
//!
//! ## Executable Code Tests
//! - `defaults_test.rs` - Tests for the `default_abi()` function and Duration constants
//! - `patterns_test.rs` - Tests for lazy_static Regex pattern initialization and matching
//! - `mod_test.rs` - Tests for module re-export functionality
//!
//! ## Validation Tests
//! - `validation_test.rs` - Comprehensive validation of constant values, ranges, and relationships
//!
//! # Test Organization
//!
//! Tests are organized by the type of code being tested:
//! - **Function tests** - Test actual executable functions
//! - **Pattern tests** - Test regex patterns and their runtime behavior
//! - **Re-export tests** - Test module system functionality
//! - **Validation tests** - Test constant value consistency and logical relationships
//!
//! # Running Tests
//!
//! ```bash
//! # Run all constants tests
//! cargo test --test constants
//!
//! # Run specific test module
//! cargo test --test constants::defaults_test
//! cargo test --test constants::patterns_test
//! cargo test --test constants::validation_test
//! ```

pub mod defaults_test;
pub mod patterns_test;
pub mod mod_test;
pub mod validation_test;