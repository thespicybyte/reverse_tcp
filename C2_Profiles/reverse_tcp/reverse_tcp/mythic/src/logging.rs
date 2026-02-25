use std::{env, fs};
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{self, fmt, layer::SubscriberExt, Registry};

pub fn init_logger(log_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let log_level = env::var("MYTHIC_DEBUG_LEVEL").unwrap_or_else(|_| "info".to_string());
    let _ = fs::create_dir_all("logs");
    let log_file = log_name.to_string();

    let file_appender = tracing_appender::rolling::daily("logs", log_file);
    let file_layer = fmt::layer().with_writer(file_appender).with_ansi(false);
    let stdout_layer = fmt::layer().with_writer(std::io::stdout).with_ansi(true);

    // Create a subscriber with explicit level configuration to ensure messages are shown
    let subscriber = Registry::default()
        .with(EnvFilter::new(log_level))
        .with(stdout_layer)
        .with(file_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    info!("Logger initialized for reverse_tcp profile");
    Ok(())
}
