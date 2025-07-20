//! Command execution abstraction for testability
//!
//! This module provides a trait-based abstraction over command execution,
//! allowing for easy mocking in tests while maintaining the same behavior
//! in production code.

use anyhow::Result;
use async_trait::async_trait;

/// Trait for executing external commands
///
/// This abstraction allows dependency injection of command execution logic,
/// making it possible to mock external command calls in tests.
#[async_trait]
pub trait CommandExecutor: Send + Sync {
    /// Execute a command and return its output
    async fn run(&self, command: &std::path::Path, args: &[&str]) -> Result<String>;

    /// Spawn a command and return its process ID
    async fn spawn(&self, command: &std::path::Path, args: &[&str]) -> Result<u32>;

    /// Execute a command with retry logic
    async fn run_with_retry(
        &self,
        command: &std::path::Path,
        args: &[&str],
        retries: u32,
    ) -> Result<String>;

    /// Execute a command, ignoring specific error patterns
    async fn run_ignoring_errors(
        &self,
        command: &std::path::Path,
        args: &[&str],
        ignore_patterns: &[&str],
    ) -> Result<String>;
}

/// Implementation of CommandExecutor for the actual CommandRunner
#[async_trait]
impl CommandExecutor for crate::utils::command::CommandRunner {
    async fn run(&self, command: &std::path::Path, args: &[&str]) -> Result<String> {
        self.run(command, args).await
    }

    async fn spawn(&self, command: &std::path::Path, args: &[&str]) -> Result<u32> {
        self.spawn(command, args).await
    }

    async fn run_with_retry(
        &self,
        command: &std::path::Path,
        args: &[&str],
        retries: u32,
    ) -> Result<String> {
        self.run_with_retry(command, args, retries).await
    }

    async fn run_ignoring_errors(
        &self,
        command: &std::path::Path,
        args: &[&str],
        ignore_patterns: &[&str],
    ) -> Result<String> {
        self.run_ignoring_errors(command, args, ignore_patterns)
            .await
    }
}

pub mod mock {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    /// Mock implementation of CommandExecutor for testing
    type CallHistory = Arc<Mutex<Vec<(String, Vec<String>)>>>;

    #[derive(Clone)]
    pub struct MockCommandExecutor {
        responses: Arc<Mutex<HashMap<String, Result<String, String>>>>,
        spawn_responses: Arc<Mutex<HashMap<String, u32>>>,
        call_history: CallHistory,
    }

    impl MockCommandExecutor {
        /// Create a new mock executor
        pub fn new() -> Self {
            Self {
                responses: Arc::new(Mutex::new(HashMap::new())),
                spawn_responses: Arc::new(Mutex::new(HashMap::new())),
                call_history: Arc::new(Mutex::new(Vec::new())),
            }
        }

        /// Add a response for a specific command
        pub fn with_response(self, command: &str, args: &[&str], response: Result<String>) -> Self {
            let key = format!("{} {}", command, args.join(" "));
            let mock_response = response.map_err(|e| e.to_string());
            self.responses.lock().unwrap().insert(key, mock_response);
            self
        }

        /// Add a successful response for a specific command
        pub fn with_success(self, command: &str, args: &[&str], output: &str) -> Self {
            self.with_response(command, args, Ok(output.to_string()))
        }

        /// Add an error response for a specific command
        pub fn with_error(self, command: &str, args: &[&str], error: &str) -> Self {
            self.with_response(command, args, Err(anyhow::anyhow!(error.to_string())))
        }

        /// Add a spawn response
        pub fn with_spawn_response(self, command: &str, args: &[&str], pid: u32) -> Self {
            let key = format!("{} {}", command, args.join(" "));
            self.spawn_responses.lock().unwrap().insert(key, pid);
            self
        }

        /// Get the call history
        pub fn call_history(&self) -> Vec<(String, Vec<String>)> {
            self.call_history.lock().unwrap().clone()
        }

        /// Clear the call history
        pub fn clear_history(&self) {
            self.call_history.lock().unwrap().clear();
        }
    }

    impl Default for MockCommandExecutor {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl CommandExecutor for MockCommandExecutor {
        async fn run(&self, command: &std::path::Path, args: &[&str]) -> Result<String> {
            // Convert Path to string, falling back to lossy conversion for non-UTF8 paths
            let command_string = command.to_string_lossy();
            let command_str = command.to_str().unwrap_or_else(|| command_string.as_ref());
            let key = format!("{} {}", command_str, args.join(" "));

            // Record the call
            self.call_history.lock().unwrap().push((
                command_str.to_string(),
                args.iter().map(|s| s.to_string()).collect(),
            ));

            // First try exact match
            let responses = self.responses.lock().unwrap();
            if let Some(response) = responses.get(&key) {
                return response.clone().map_err(|e| anyhow::anyhow!(e));
            }

            // If no exact match, try matching by command basename
            let command_basename = command
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(command_str);
            let basename_key = format!("{} {}", command_basename, args.join(" "));

            responses
                .get(&basename_key)
                .cloned()
                .unwrap_or_else(|| Err(format!("No mock response for: {key}")))
                .map_err(|e| anyhow::anyhow!(e))
        }

        async fn spawn(&self, command: &std::path::Path, args: &[&str]) -> Result<u32> {
            // Convert Path to string, falling back to lossy conversion for non-UTF8 paths
            let command_string = command.to_string_lossy();
            let command_str = command.to_str().unwrap_or_else(|| command_string.as_ref());
            let key = format!("{} {}", command_str, args.join(" "));

            // Record the call
            self.call_history.lock().unwrap().push((
                command_str.to_string(),
                args.iter().map(|s| s.to_string()).collect(),
            ));

            // First try exact match
            let spawn_responses = self.spawn_responses.lock().unwrap();
            if let Some(&pid) = spawn_responses.get(&key) {
                return Ok(pid);
            }

            // If no exact match, try matching by command basename
            let command_basename = command
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(command_str);
            let basename_key = format!("{} {}", command_basename, args.join(" "));

            spawn_responses
                .get(&basename_key)
                .copied()
                .ok_or_else(|| anyhow::anyhow!("No mock spawn response for: {key}"))
        }

        async fn run_with_retry(
            &self,
            command: &std::path::Path,
            args: &[&str],
            _retries: u32,
        ) -> Result<String> {
            // For simplicity, just delegate to run in the mock
            self.run(command, args).await
        }

        async fn run_ignoring_errors(
            &self,
            command: &std::path::Path,
            args: &[&str],
            _ignore_patterns: &[&str],
        ) -> Result<String> {
            // For simplicity, just delegate to run in the mock
            self.run(command, args).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::MockCommandExecutor;
    use super::*;

    #[tokio::test]
    async fn test_mock_executor_success() {
        let executor = MockCommandExecutor::new().with_success("echo", &["hello"], "hello\n");

        let result = executor.run(std::path::Path::new("echo"), &["hello"]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello\n");

        // Check call history
        let history = executor.call_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].0, "echo");
        assert_eq!(history[0].1, vec!["hello"]);
    }

    #[tokio::test]
    async fn test_mock_executor_error() {
        let executor = MockCommandExecutor::new().with_error("false", &[], "Command failed");

        let result = executor.run(std::path::Path::new("false"), &[]).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Command failed"));
    }

    #[tokio::test]
    async fn test_mock_executor_spawn() {
        let executor = MockCommandExecutor::new().with_spawn_response("sleep", &["10"], 12345);

        let result = executor.spawn(std::path::Path::new("sleep"), &["10"]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 12345);
    }

    #[tokio::test]
    async fn test_mock_executor_non_utf8_path() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        // Create a path with non-UTF8 bytes (only works on Unix)
        #[cfg(unix)]
        {
            let non_utf8_bytes = vec![0x80, 0x81, 0x82]; // Invalid UTF-8
            let non_utf8_path = std::path::Path::new(OsStr::from_bytes(&non_utf8_bytes));

            let executor = MockCommandExecutor::new();
            // The mock should handle non-UTF8 paths gracefully using lossy conversion
            let result = executor.run(non_utf8_path, &["test"]).await;
            assert!(result.is_err()); // Will fail because no mock response configured

            // But it shouldn't panic, and the call should be recorded
            let history = executor.call_history();
            assert_eq!(history.len(), 1);
        }
    }
}
