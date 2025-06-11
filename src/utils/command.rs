//! Command execution utilities
//!
//! This module provides a unified interface for executing external commands
//! asynchronously. It handles command execution, output capture, error handling,
//! and debug logging in a consistent manner across the application.

use anyhow::{Context, Result};
use std::ffi::OsStr;
use tokio::process::Command;

/// A utility for executing external commands asynchronously.
///
/// CommandRunner provides a consistent interface for running external tools
/// like Android SDK utilities, iOS simulator commands, and other system tools.
/// It handles output capture, error propagation, and optional debug logging.
///
/// # Examples
/// ```rust
/// let runner = CommandRunner::new();
/// let output = runner.run("adb", &["devices"]).await?;
/// println!("Connected devices: {}", output);
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
    /// ```rust
    /// let runner = CommandRunner::new();
    /// let devices = runner.run("adb", &["devices"]).await?;
    /// let avds = runner.run("avdmanager", &["list", "avd"]).await?;
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
            eprintln!("[DEBUG] Executing command: {}", command_str);
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
            eprintln!("[DEBUG] Command stdout: {}", stdout);
            eprintln!("[DEBUG] Command stderr: {}", stderr);
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
    /// ```rust
    /// let runner = CommandRunner::new();
    /// let pid = runner.spawn("open", &["-a", "Simulator"]).await?;
    /// println!("Launched Simulator with PID: {}", pid);
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
}
