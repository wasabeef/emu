//! Emu - A lazygit-inspired TUI for managing mobile device emulators/simulators

use anyhow::Result;
use clap::Parser;
use emu::app::App;

#[derive(Parser)]
#[command(
    name = "emu",
    version,
    about = "A lazygit-inspired TUI for managing Android emulators and iOS simulators"
)]
struct Cli {
    #[arg(short, long, default_value = "info")]
    log_level: String,

    #[arg(long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    color_eyre::install().map_err(|e| anyhow::anyhow!("Failed to install color_eyre: {}", e))?;

    // Only enable console logging in debug mode
    if cli.debug {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&cli.log_level))
            .format_timestamp_secs()
            .init();
    } else {
        // In normal TUI mode, suppress Android emulator console output
        std::env::set_var("ANDROID_EMULATOR_LOG_ENABLE", "0");
        std::env::set_var("ANDROID_AVD_VERBOSE", "0");
        std::env::set_var("ANDROID_VERBOSE", "0");
    }
    // In normal TUI mode, logging is disabled to prevent console output

    run_tui().await
}

async fn run_tui() -> Result<()> {
    use crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{backend::CrosstermBackend, Terminal};
    use std::io;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    // Run app
    let app = App::new().await?;
    let result = app.run(terminal).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    result
}
