//! Command execution utilities
//!
//! This module provides a unified interface for executing external commands
//! asynchronously. It handles command execution, output capture, error handling,
//! and debug logging in a consistent manner across the application.

use anyhow::{Context, Result};
use std::ffi::OsStr;
use tokio::process::Command;

use crate::constants::timeouts::{INITIAL_RETRY_DELAY, MAX_RETRY_DELAY};

/// A utility for executing external commands asynchronously.
///
/// CommandRunner provides a consistent interface for running external tools
/// like Android SDK utilities, iOS simulator commands, and other system tools.
/// It handles output capture, error propagation, and optional debug logging.
///
/// # Examples
/// ```rust,no_run
/// use emu::utils::CommandRunner;
///
/// # async fn example() -> anyhow::Result<()> {
/// let runner = CommandRunner::new();
/// let output = runner.run("adb", &["devices"]).await?;
/// println!("Connected devices: {}", output);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct CommandRunner;

impl Default for CommandRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRunner {
    /// Creates a new CommandRunner instance.
    ///
    /// # Returns
    /// A new CommandRunner ready to execute commands
    pub fn new() -> Self {
        Self
    }

    /// Executes a command and waits for it to complete, returning stdout.
    ///
    /// This method runs the specified command with arguments and captures
    /// both stdout and stderr. If the command fails (non-zero exit code),
    /// an error is returned with details from stderr and stdout.
    ///
    /// Debug logging is enabled when `RUST_LOG=debug` environment variable is set,
    /// which will print the command being executed and its output to stderr.
    ///
    /// # Arguments
    /// * `program` - The command/program to execute
    /// * `args` - Iterator of arguments to pass to the command
    ///
    /// # Returns
    /// * `Ok(String)` - Command stdout if execution succeeds
    /// * `Err(anyhow::Error)` - If command fails or cannot be executed
    ///
    /// # Examples
    /// ```rust,no_run
    /// use emu::utils::CommandRunner;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let runner = CommandRunner::new();
    /// let devices = runner.run("adb", &["devices"]).await?;
    /// let avds = runner.run("avdmanager", &["list", "avd"]).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Error Handling
    /// Errors can occur for several reasons:
    /// - Command not found in PATH
    /// - Command execution failure (non-zero exit code)
    /// - Permission denied
    /// - Invalid arguments
    pub async fn run<S, I, A>(&self, program: S, args: I) -> Result<String>
    where
        S: AsRef<OsStr>,
        I: IntoIterator<Item = A>,
        A: AsRef<OsStr>,
    {
        let program_ref = program.as_ref();
        let args_vec: Vec<_> = args
            .into_iter()
            .map(|a| a.as_ref().to_os_string())
            .collect();

        // Debug logging only when RUST_LOG=debug is set
        if std::env::var("RUST_LOG")
            .unwrap_or_default()
            .contains("debug")
        {
            let command_str = format!(
                "{} {}",
                program_ref.to_string_lossy(),
                args_vec
                    .iter()
                    .map(|a| a.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            eprintln!("[DEBUG] Executing command: {command_str}");
        }

        let output = Command::new(program_ref)
            .args(&args_vec)
            .output()
            .await
            .context("Failed to execute command")?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Debug logging only when RUST_LOG=debug is set
        if std::env::var("RUST_LOG")
            .unwrap_or_default()
            .contains("debug")
        {
            eprintln!("[DEBUG] Command exit code: {:?}", output.status.code());
            eprintln!("[DEBUG] Command stdout: {stdout}");
            eprintln!("[DEBUG] Command stderr: {stderr}");
        }

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Command failed with exit code {}: stderr: {} stdout: {}",
                output.status.code().unwrap_or(-1),
                stderr.trim(),
                stdout.trim()
            ));
        }

        Ok(stdout.into_owned())
    }

    /// Spawns a command in the background and returns immediately.
    ///
    /// This method starts a command as a detached background process
    /// without waiting for it to complete. All stdio streams are
    /// redirected to null to prevent output interference.
    ///
    /// This is useful for launching GUI applications or long-running
    /// processes that should continue independently of the main application.
    ///
    /// # Arguments
    /// * `program` - The command/program to spawn
    /// * `args` - Iterator of arguments to pass to the command
    ///
    /// # Returns
    /// * `Ok(u32)` - Process ID of the spawned command
    /// * `Err(anyhow::Error)` - If the command cannot be spawned
    ///
    /// # Examples
    /// ```rust,no_run
    /// use emu::utils::CommandRunner;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let runner = CommandRunner::new();
    /// let pid = runner.spawn("open", &["-a", "Simulator"]).await?;
    /// println!("Launched Simulator with PID: {}", pid);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Note
    /// The spawned process runs independently and its exit status
    /// is not monitored. Use `run()` if you need to capture output
    /// or wait for completion.
    pub async fn spawn<S, I, A>(&self, program: S, args: I) -> Result<u32>
    where
        S: AsRef<OsStr>,
        I: IntoIterator<Item = A>,
        A: AsRef<OsStr>,
    {
        let child = Command::new(program)
            .args(args)
            .stdout(std::process::Stdio::null()) // Suppress stdout output
            .stderr(std::process::Stdio::null()) // Suppress stderr output
            .stdin(std::process::Stdio::null()) // No stdin needed
            .spawn()
            .context("Failed to spawn command")?;

        Ok(child.id().unwrap_or(0))
    }

    /// Executes a command ignoring specific error patterns (useful for "already in state" errors).
    ///
    /// This method runs a command and only returns an error if it doesn't match
    /// any of the provided ignore patterns. This is particularly useful for
    /// platform commands that return errors when a device is already in the
    /// requested state.
    ///
    /// # Arguments
    /// * `program` - The command/program to execute
    /// * `args` - Iterator of arguments to pass to the command
    /// * `ignore_patterns` - Patterns to ignore in error messages
    ///
    /// # Returns
    /// * `Ok(String)` - Command stdout if execution succeeds or error is ignored
    /// * `Err(anyhow::Error)` - If command fails with an error not in ignore patterns
    ///
    /// # Examples
    /// ```rust,no_run
    /// use emu::utils::CommandRunner;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let runner = CommandRunner::new();
    /// let device_id = "12345-6789";
    /// // Ignore "already booted" errors when starting iOS simulator
    /// runner.run_ignoring_errors(
    ///     "xcrun",
    ///     &["simctl", "boot", device_id],
    ///     &["Unable to boot device in current state: Booted"]
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run_ignoring_errors<S, I, A>(
        &self,
        program: S,
        args: I,
        ignore_patterns: &[&str],
    ) -> Result<String>
    where
        S: AsRef<OsStr>,
        I: IntoIterator<Item = A>,
        A: AsRef<OsStr>,
    {
        match self.run(program, args).await {
            Ok(output) => Ok(output),
            Err(e) => {
                let error_msg = e.to_string();

                // Check if error matches any ignore pattern
                for pattern in ignore_patterns {
                    if error_msg.contains(pattern) {
                        // Log that we're ignoring this error
                        log::info!("Ignoring expected error: {error_msg}");
                        return Ok(String::new());
                    }
                }

                // Re-throw the error if it doesn't match any pattern
                Err(e)
            }
        }
    }

    /// Executes a command with retry logic for transient failures.
    ///
    /// This method attempts to run a command multiple times with exponential
    /// backoff between attempts. Useful for commands that may fail due to
    /// temporary resource contention or network issues.
    ///
    /// # Arguments
    /// * `program` - The command/program to execute
    /// * `args` - Iterator of arguments to pass to the command
    /// * `max_retries` - Maximum number of retry attempts (0 = no retries)
    ///
    /// # Returns
    /// * `Ok(String)` - Command stdout if any attempt succeeds
    /// * `Err(anyhow::Error)` - If all attempts fail
    ///
    /// # Examples
    /// ```rust,no_run
    /// use emu::utils::CommandRunner;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let runner = CommandRunner::new();
    /// // Try up to 3 times to list devices
    /// let devices = runner.run_with_retry("adb", &["devices"], 2).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run_with_retry<S, I, A>(
        &self,
        program: S,
        args: I,
        max_retries: u32,
    ) -> Result<String>
    where
        S: AsRef<OsStr>,
        I: IntoIterator<Item = A> + Clone,
        A: AsRef<OsStr>,
    {
        let mut last_error = None;
        let mut delay = INITIAL_RETRY_DELAY;

        for attempt in 0..=max_retries {
            match self.run(program.as_ref(), args.clone()).await {
                Ok(output) => return Ok(output),
                Err(e) => {
                    last_error = Some(e);

                    if attempt < max_retries {
                        log::warn!(
                            "Command failed (attempt {}/{}), retrying after {:?}",
                            attempt + 1,
                            max_retries + 1,
                            delay
                        );
                        tokio::time::sleep(delay).await;

                        // Exponential backoff with max delay
                        delay = std::cmp::min(delay * 2, MAX_RETRY_DELAY);
                    }
                }
            }
        }

        Err(last_error.unwrap())
            .context(format!("Command failed after {} attempts", max_retries + 1))
    }
}
