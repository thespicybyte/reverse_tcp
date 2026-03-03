use std::{env, fs, fs::OpenOptions};
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, Registry};

pub fn init_logger(log_dir: &str, log_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenv::dotenv();

    let log_level = env::var("DEBUG_LEVEL").unwrap_or_else(|_| "info".to_string());

    // Ensure logs directory exists
    fs::create_dir_all("logs")?;

    // Open (or create) a single log file and append to it
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{}/{}", log_dir, log_name))?;

    let file_layer = fmt::layer().with_writer(file).with_ansi(false);

    let stdout_layer = fmt::layer().with_writer(std::io::stdout).with_ansi(true);

    let subscriber = Registry::default()
        .with(EnvFilter::new(log_level))
        .with(stdout_layer)
        .with(file_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    info!("Logger initialized for reverse_tcp profile");

    Ok(())
}
