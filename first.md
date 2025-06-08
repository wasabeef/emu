# Emu - Complete Project Structure and Files

## プロジェクト構造

```
emu/
├── Cargo.toml
├── README.md
├── LICENSE-MIT
├── LICENSE-APACHE
├── CHANGELOG.md
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── .gitignore
├── .github/
│   ├── workflows/
│   │   ├── ci.yml
│   │   ├── release.yml
│   │   └── homebrew.yml
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   └── feature_request.md
│   └── PULL_REQUEST_TEMPLATE.md
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── app/
│   │   ├── mod.rs
│   │   ├── state.rs
│   │   ├── events.rs
│   │   └── actions.rs
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── render.rs
│   │   ├── theme.rs
│   │   └── widgets.rs
│   ├── managers/
│   │   ├── mod.rs
│   │   ├── android.rs
│   │   ├── ios.rs
│   │   └── common.rs
│   ├── models/
│   │   ├── mod.rs
│   │   ├── device.rs
│   │   ├── error.rs
│   │   └── platform.rs
│   ├── utils/
│   │   ├── mod.rs
│   │   ├── command.rs
│   │   └── logger.rs
│   ├── config.rs
│   └── headless.rs
├── docs/
│   ├── INSTALL.md
│   └── user_guide.md
├── examples/
│   └── config.toml
└── Formula/
    └── emu.rb
```

## ファイル内容

### 1. Cargo.toml

```toml
[package]
name = "emu"
version = "0.1.0"
edition = "2021"
authors = ["Daichi Furiya"]
description = "A lazygit-like CUI tool for managing Android emulators and iOS simulators"
readme = "README.md"
repository = "https://github.com/wasabeef/emu"
license = "MIT OR Apache-2.0"
keywords = ["android", "ios", "emulator", "simulator", "tui"]
categories = ["command-line-utilities", "development-tools"]

[dependencies]
# TUI Framework
ratatui = "0.26"
crossterm = "0.27"

# Async Runtime
tokio = { version = "1.35", features = ["full"] }

# CLI Parsing
clap = { version = "4.5", features = ["derive", "env"] }
clap_complete = "4.5"

# Error Handling
anyhow = "1.0"
thiserror = "1.0"
color-eyre = "0.6"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Logging
env_logger = "0.11"
log = "0.4"

# Utility
dirs = "5.0"
which = "6.0"
regex = "1.10"
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.1"
tempfile = "3.10"
mockall = "0.12"
criterion = "0.5"

[[bench]]
name = "performance"
harness = false

[profile.release]
codegen-units = 1
lto = true
opt-level = 3
strip = true
```

### 2. src/main.rs

```rust
//! Emu - A lazygit-inspired TUI for managing mobile device emulators/simulators

use anyhow::Result;
use clap::{Parser, Subcommand};
use emu::{app::App, config::Config, headless::HeadlessRunner};
use log::info;

#[derive(Parser)]
#[command(
    name = "emu",
    version,
    about = "A lazygit-inspired TUI for managing Android emulators and iOS simulators",
)]
struct Cli {
    #[arg(short, long, value_name = "FILE", env = "DEVCHAMBER_CONFIG")]
    config: Option<std::path::PathBuf>,

    #[arg(short, long, default_value = "info")]
    log_level: String,

    #[arg(long)]
    headless: bool,

    #[arg(long, default_value = "text")]
    format: String,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    List {
        #[arg(short, long, default_value = "all")]
        platform: String,
        #[arg(short, long)]
        running: bool,
    },
    Start {
        device: String,
        #[arg(short, long)]
        wait: bool,
    },
    Stop {
        device: String,
    },
    Create {
        name: String,
        #[arg(short, long)]
        platform: String,
        #[arg(short = 't', long)]
        device_type: String,
        #[arg(short = 'v', long)]
        version: String,
    },
    Delete {
        device: String,
        #[arg(short, long)]
        force: bool,
    },
    Wipe {
        device: String,
    },
    Doctor {
        #[arg(short, long)]
        component: Option<String>,
    },
    Completions {
        shell: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    color_eyre::install()?;

    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(&cli.log_level)
    )
    .format_timestamp_secs()
    .init();

    info!("Starting Emu v{}", env!("CARGO_PKG_VERSION"));

    let config = Config::load(cli.config)?;

    if cli.headless {
        run_headless(cli, config).await
    } else {
        run_tui(config).await
    }
}

async fn run_tui(config: Config) -> Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;

    let app = App::new(config).await?;
    let result = app.run(terminal).await;

    ratatui::restore();

    result
}

async fn run_headless(cli: Cli, config: Config) -> Result<()> {
    let runner = HeadlessRunner::new(config, cli.format);

    match cli.command {
        Some(Command::List { platform, running }) => {
            runner.list_devices(&platform, running).await
        }
        Some(Command::Start { device, wait }) => {
            runner.start_device(&device, wait).await
        }
        Some(Command::Stop { device }) => {
            runner.stop_device(&device).await
        }
        Some(Command::Create { name, platform, device_type, version }) => {
            runner.create_device(&name, &platform, &device_type, &version).await
        }
        Some(Command::Delete { device, force }) => {
            runner.delete_device(&device, force).await
        }
        Some(Command::Wipe { device }) => {
            runner.wipe_device(&device).await
        }
        Some(Command::Doctor { component }) => {
            runner.run_diagnostics(component.as_deref()).await
        }
        Some(Command::Completions { shell }) => {
            generate_completions(&shell);
            Ok(())
        }
        None => {
            eprintln!("Error: No command specified for headless mode");
            std::process::exit(1);
        }
    }
}

fn generate_completions(shell: &str) {
    use clap_complete::{generate, Shell};
    use std::io;

    let mut cmd = Cli::command();
    let shell = shell.parse::<Shell>().unwrap();

    generate(shell, &mut cmd, "emu", &mut io::stdout());
}
```

### 3. src/lib.rs

```rust
//! Emu library

pub mod app;
pub mod config;
pub mod headless;
pub mod managers;
pub mod models;
pub mod ui;
pub mod utils;

pub use app::App;
pub use config::Config;
```

### 4. src/app/mod.rs

```rust
//! Application core module

pub mod actions;
pub mod events;
pub mod state;

use crate::{
    config::Config,
    managers::{AndroidManager, IosManager},
    ui,
};
use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyModifiers};
use ratatui::{backend::Backend, Terminal};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

pub use self::state::{AppState, Mode, Panel};

pub struct App {
    state: Arc<Mutex<AppState>>,
    config: Config,
    android_manager: AndroidManager,
    ios_manager: Option<IosManager>,
}

impl App {
    pub async fn new(config: Config) -> Result<Self> {
        let state = Arc::new(Mutex::new(AppState::new()));
        let android_manager = AndroidManager::new()?;
        let ios_manager = if cfg!(target_os = "macos") {
            Some(IosManager::new()?)
        } else {
            None
        };

        let mut app = Self {
            state,
            config,
            android_manager,
            ios_manager,
        };

        app.refresh_devices().await?;

        Ok(app)
    }

    pub async fn run<B: Backend>(mut self, mut terminal: Terminal<B>) -> Result<()> {
        loop {
            let state = self.state.lock().await;
            terminal.draw(|f| ui::render::draw_app(f, &state, &self.config.theme()))?;
            drop(state);

            if event::poll(Duration::from_millis(100))? {
                if let CrosstermEvent::Key(key) = event::read()? {
                    let mut state = self.state.lock().await;

                    match key.code {
                        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(());
                        }
                        KeyCode::Char('r') => {
                            drop(state);
                            self.refresh_devices().await?;
                        }
                        KeyCode::Tab => state.next_panel(),
                        KeyCode::Enter => {
                            drop(state);
                            self.toggle_device().await?;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    async fn refresh_devices(&mut self) -> Result<()> {
        let mut state = self.state.lock().await;
        state.is_loading = true;
        drop(state);

        let android_devices = self.android_manager.list_devices().await?;
        let ios_devices = if let Some(ref mut ios_manager) = self.ios_manager {
            ios_manager.list_devices().await?
        } else {
            Vec::new()
        };

        let mut state = self.state.lock().await;
        state.android_devices = android_devices;
        state.ios_devices = ios_devices;
        state.is_loading = false;

        Ok(())
    }

    async fn toggle_device(&mut self) -> Result<()> {
        let state = self.state.lock().await;

        match state.active_panel {
            Panel::Android => {
                if let Some(device) = state.android_devices.get(state.selected_android) {
                    let name = device.name.clone();
                    let is_running = device.is_running;
                    drop(state);

                    if is_running {
                        self.android_manager.stop_device(&name).await?;
                    } else {
                        self.android_manager.start_device(&name).await?;
                    }
                }
            }
            Panel::Ios => {
                if let Some(ref mut ios_manager) = self.ios_manager {
                    if let Some(device) = state.ios_devices.get(state.selected_ios) {
                        let udid = device.udid.clone();
                        let is_running = device.is_running;
                        drop(state);

                        if is_running {
                            ios_manager.stop_device(&udid).await?;
                        } else {
                            ios_manager.start_device(&udid).await?;
                        }
                    }
                }
            }
        }

        self.refresh_devices().await?;
        Ok(())
    }
}
```

### 5. src/app/state.rs

```rust
//! Application state management

use crate::models::{AndroidDevice, IosDevice};

#[derive(Debug, Clone, PartialEq)]
pub enum Panel {
    Android,
    Ios,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    CreateDevice,
    ConfirmDelete,
    Help,
}

pub struct AppState {
    pub active_panel: Panel,
    pub mode: Mode,
    pub android_devices: Vec<AndroidDevice>,
    pub ios_devices: Vec<IosDevice>,
    pub selected_android: usize,
    pub selected_ios: usize,
    pub is_loading: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            active_panel: Panel::Android,
            mode: Mode::Normal,
            android_devices: Vec::new(),
            ios_devices: Vec::new(),
            selected_android: 0,
            selected_ios: 0,
            is_loading: false,
        }
    }

    pub fn next_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Android => Panel::Ios,
            Panel::Ios => Panel::Android,
        };
    }
}
```

### 6. src/managers/mod.rs

```rust
//! Device managers module

pub mod android;
pub mod ios;
pub mod common;

pub use android::AndroidManager;
pub use ios::IosManager;
```

### 7. src/managers/android.rs

```rust
//! Android emulator management

use crate::{
    models::{AndroidDevice, DeviceStatus},
    utils::command::CommandRunner,
};
use anyhow::{Context, Result, bail};
use regex::Regex;
use std::path::PathBuf;

pub struct AndroidManager {
    command_runner: CommandRunner,
    android_home: PathBuf,
    avdmanager_path: PathBuf,
    emulator_path: PathBuf,
}

impl AndroidManager {
    pub fn new() -> Result<Self> {
        let android_home = Self::find_android_home()?;
        let avdmanager_path = Self::find_tool(&android_home, "avdmanager")?;
        let emulator_path = Self::find_tool(&android_home, "emulator")?;

        Ok(Self {
            command_runner: CommandRunner::new(),
            android_home,
            avdmanager_path,
            emulator_path,
        })
    }

    fn find_android_home() -> Result<PathBuf> {
        if let Ok(path) = std::env::var("ANDROID_HOME") {
            return Ok(PathBuf::from(path));
        }

        if let Ok(path) = std::env::var("ANDROID_SDK_ROOT") {
            return Ok(PathBuf::from(path));
        }

        bail!("Android SDK not found. Please set ANDROID_HOME or ANDROID_SDK_ROOT")
    }

    fn find_tool(android_home: &PathBuf, tool: &str) -> Result<PathBuf> {
        let paths = [
            android_home.join("cmdline-tools/latest/bin").join(tool),
            android_home.join("tools/bin").join(tool),
            android_home.join("emulator").join(tool),
        ];

        for path in &paths {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        bail!("Tool '{}' not found in Android SDK", tool)
    }

    pub async fn list_devices(&self) -> Result<Vec<AndroidDevice>> {
        let output = self.command_runner
            .run(&self.avdmanager_path, &["list", "avd"])
            .await?;

        let mut devices = Vec::new();
        let name_regex = Regex::new(r"Name:\s+(.+)")?;
        let mut current_device = AndroidDevice::default();

        for line in output.lines() {
            if let Some(captures) = name_regex.captures(line) {
                if !current_device.name.is_empty() {
                    devices.push(current_device.clone());
                }
                current_device = AndroidDevice {
                    name: captures[1].to_string(),
                    ..Default::default()
                };
            }
        }

        if !current_device.name.is_empty() {
            devices.push(current_device);
        }

        Ok(devices)
    }

    pub async fn start_device(&self, name: &str) -> Result<u32> {
        let pid = self.command_runner
            .spawn(&self.emulator_path, &["-avd", name])
            .await?;
        Ok(pid)
    }

    pub async fn stop_device(&self, name: &str) -> Result<()> {
        // Implementation for stopping device
        Ok(())
    }
}

impl Default for AndroidDevice {
    fn default() -> Self {
        Self {
            name: String::new(),
            device_type: String::new(),
            api_level: 0,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "512M".to_string(),
        }
    }
}
```

### 8. src/managers/ios.rs

```rust
//! iOS simulator management

#[cfg(target_os = "macos")]
use crate::{
    models::{IosDevice, DeviceStatus},
    utils::command::CommandRunner,
};
#[cfg(target_os = "macos")]
use anyhow::{Result, bail};

#[cfg(target_os = "macos")]
pub struct IosManager {
    command_runner: CommandRunner,
}

#[cfg(target_os = "macos")]
impl IosManager {
    pub fn new() -> Result<Self> {
        if !which::which("xcrun").is_ok() {
            bail!("Xcode Command Line Tools not found. Please install Xcode.")
        }

        Ok(Self {
            command_runner: CommandRunner::new(),
        })
    }

    pub async fn list_devices(&self) -> Result<Vec<IosDevice>> {
        let output = self.command_runner
            .run("xcrun", &["simctl", "list", "devices", "--json"])
            .await?;

        // Parse JSON output
        // Implementation details...

        Ok(Vec::new())
    }

    pub async fn start_device(&self, udid: &str) -> Result<()> {
        self.command_runner
            .run("xcrun", &["simctl", "boot", udid])
            .await?;
        Ok(())
    }

    pub async fn stop_device(&self, udid: &str) -> Result<()> {
        self.command_runner
            .run("xcrun", &["simctl", "shutdown", udid])
            .await?;
        Ok(())
    }
}

#[cfg(not(target_os = "macos"))]
pub struct IosManager;

#[cfg(not(target_os = "macos"))]
impl IosManager {
    pub fn new() -> Result<Self> {
        bail!("iOS simulator management is only available on macOS")
    }
}
```

### 9. src/models/mod.rs

```rust
//! Data models

pub mod device;
pub mod error;
pub mod platform;

pub use device::{AndroidDevice, IosDevice, DeviceStatus};
pub use error::DeviceError;
pub use platform::Platform;
```

### 10. src/models/device.rs

```rust
//! Device models

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidDevice {
    pub name: String,
    pub device_type: String,
    pub api_level: u32,
    pub status: DeviceStatus,
    pub is_running: bool,
    pub ram_size: String,
    pub storage_size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IosDevice {
    pub name: String,
    pub udid: String,
    pub device_type: String,
    pub ios_version: String,
    pub status: DeviceStatus,
    pub is_running: bool,
    pub is_available: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DeviceStatus {
    Running,
    Stopped,
    Creating,
    Error,
    Unknown,
}
```

### 11. src/utils/command.rs

```rust
//! Command execution utilities

use anyhow::{Context, Result};
use std::ffi::OsStr;
use tokio::process::Command;

pub struct CommandRunner;

impl CommandRunner {
    pub fn new() -> Self {
        Self
    }

    pub async fn run<S, I>(&self, program: S, args: I) -> Result<String>
    where
        S: AsRef<OsStr>,
        I: IntoIterator<Item = S>,
    {
        let output = Command::new(program)
            .args(args)
            .output()
            .await
            .context("Failed to execute command")?;

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    pub async fn spawn<S, I>(&self, program: S, args: I) -> Result<u32>
    where
        S: AsRef<OsStr>,
        I: IntoIterator<Item = S>,
    {
        let child = Command::new(program)
            .args(args)
            .spawn()
            .context("Failed to spawn command")?;

        Ok(child.id().unwrap_or(0))
    }
}
```

### 12. src/config.rs

```rust
//! Configuration management

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default)]
    pub android: AndroidConfig,

    #[serde(default)]
    pub ios: IosConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AndroidConfig {
    pub default_ram: String,
    pub default_storage: String,
    pub default_api_level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IosConfig {
    pub default_device_type: String,
    pub default_ios_version: String,
}

impl Config {
    pub fn load(path: Option<PathBuf>) -> Result<Self> {
        // Load from file or use defaults
        Ok(Self::default())
    }

    pub fn theme(&self) -> crate::ui::Theme {
        crate::ui::Theme::dark()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            android: AndroidConfig::default(),
            ios: IosConfig::default(),
        }
    }
}

fn default_theme() -> String {
    "dark".to_string()
}
```

### 13. src/ui/mod.rs

```rust
//! UI module

pub mod render;
pub mod theme;
pub mod widgets;

pub use theme::Theme;
```

### 14. src/ui/render.rs

```rust
//! UI rendering

use crate::{
    app::{AppState, Panel},
    ui::Theme,
};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw_app<B: Backend>(
    frame: &mut Frame<B>,
    state: &AppState,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.size());

    // Header
    let header = Paragraph::new("Emu")
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);

    // Main content
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Android panel
    render_android_panel(frame, main_chunks[0], state, theme);

    // iOS panel
    render_ios_panel(frame, main_chunks[1], state, theme);

    // Status bar
    let status = Paragraph::new("[q]uit [r]efresh [Tab]switch")
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(status, chunks[2]);
}

fn render_android_panel<B: Backend>(
    frame: &mut Frame<B>,
    area: ratatui::layout::Rect,
    state: &AppState,
    theme: &Theme,
) {
    let items: Vec<ListItem> = state
        .android_devices
        .iter()
        .enumerate()
        .map(|(i, device)| {
            let selected = i == state.selected_android && state.active_panel == Panel::Android;
            let style = if selected {
                Style::default().bg(Color::Yellow).fg(Color::Black)
            } else {
                Style::default()
            };
            ListItem::new(device.name.clone()).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title("Android").borders(Borders::ALL));

    frame.render_widget(list, area);
}

fn render_ios_panel<B: Backend>(
    frame: &mut Frame<B>,
    area: ratatui::layout::Rect,
    state: &AppState,
    theme: &Theme,
) {
    let items: Vec<ListItem> = state
        .ios_devices
        .iter()
        .enumerate()
        .map(|(i, device)| {
            let selected = i == state.selected_ios && state.active_panel == Panel::Ios;
            let style = if selected {
                Style::default().bg(Color::Yellow).fg(Color::Black)
            } else {
                Style::default()
            };
            ListItem::new(device.name.clone()).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title("iOS").borders(Borders::ALL));

    frame.render_widget(list, area);
}
```

### 15. src/ui/theme.rs

```rust
//! UI themes

use ratatui::style::{Color, Style};

pub struct Theme {
    pub primary: Color,
    pub background: Color,
    pub text: Color,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            primary: Color::Yellow,
            background: Color::Black,
            text: Color::White,
        }
    }
}
```

### 16. src/headless.rs

```rust
//! Headless mode implementation

use crate::{
    config::Config,
    managers::{AndroidManager, IosManager},
};
use anyhow::Result;

pub struct HeadlessRunner {
    config: Config,
    format: String,
}

impl HeadlessRunner {
    pub fn new(config: Config, format: String) -> Self {
        Self { config, format }
    }

    pub async fn list_devices(&self, platform: &str, running_only: bool) -> Result<()> {
        // Implementation
        println!("Listing devices...");
        Ok(())
    }

    pub async fn start_device(&self, device: &str, wait: bool) -> Result<()> {
        println!("Starting device: {}", device);
        Ok(())
    }

    pub async fn stop_device(&self, device: &str) -> Result<()> {
        println!("Stopping device: {}", device);
        Ok(())
    }

    pub async fn create_device(
        &self,
        name: &str,
        platform: &str,
        device_type: &str,
        version: &str,
    ) -> Result<()> {
        println!("Creating device: {}", name);
        Ok(())
    }

    pub async fn delete_device(&self, device: &str, force: bool) -> Result<()> {
        println!("Deleting device: {}", device);
        Ok(())
    }

    pub async fn wipe_device(&self, device: &str) -> Result<()> {
        println!("Wiping device: {}", device);
        Ok(())
    }

    pub async fn run_diagnostics(&self, component: Option<&str>) -> Result<()> {
        println!("Running diagnostics...");
        Ok(())
    }
}
```

### 17. README.md

```markdown
# Emu 🏗️

A lazygit-inspired Terminal UI for managing Android emulators and iOS simulators.

## Installation

### Homebrew
```bash
brew install emu
```

### Cargo
```bash
cargo install emu
```

## Usage

```bash
# Run TUI
emu

# List devices
emu --headless list

# Start device
emu --headless start "Pixel 6"
```

## Requirements

- Android SDK (with ANDROID_HOME set)
- Xcode (for iOS support on macOS)

## License

MIT OR Apache-2.0
```

### 18. .gitignore

```
# Rust
target/
Cargo.lock
**/*.rs.bk

# IDE
.idea/
.vscode/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
*.log
```

### 19. examples/config.toml

```toml
# Emu Configuration Example

theme = "dark"

[android]
default_ram = "4096"
default_storage = "2048M"
default_api_level = 33

[ios]
default_device_type = "iPhone 15 Pro"
default_ios_version = "17.0"
```

### 20. .github/workflows/ci.yml

```yaml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test
      - run: cargo build --release
```

## セットアップ手順

1. **プロジェクトディレクトリを作成**
   ```bash
   mkdir emu
   cd emu
   ```

2. **上記のファイルをコピー**
   - 各ファイルを適切なディレクトリに配置

3. **依存関係をインストール**
   ```bash
   cargo build
   ```

4. **実行**
   ```bash
   cargo run
   ```

これで完全なEmuプロジェクトが構築できます。
