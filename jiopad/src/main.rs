use jiopad::{Daemon, Config, cli, ui};
use std::process;
use tracing::{info, error};

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let args = cli::parse_args();

    // Initialize logging
    init_logging(&args);

    // Print startup banner
    let network = args.network.as_deref().unwrap_or("mainnet");
    ui::print_banner(env!("CARGO_PKG_VERSION"), network);

    // Load configuration (use defaults unless config file is provided)
    let mut config = if let Some(network) = &args.network {
        Config::for_network(network).unwrap_or_else(|_| Config::default())
    } else if let Some(config_path) = &args.config_path {
        Config::load(config_path).unwrap_or_else(|_| Config::default())
    } else {
        Config::default()
    };

    // Apply CLI overrides
    config.apply_cli_overrides(&args);

    // Print configuration summary
    ui::print_config_summary(&config);

    // Create and start daemon
    let daemon = match Daemon::new(config).await {
        Ok(d) => d,
        Err(e) => {
            ui::print_status("✗", &format!("Failed to initialize daemon: {}", e), ui::StatusType::Error);
            error!("Failed to initialize daemon: {}", e);
            process::exit(1);
        }
    };

    // Run daemon
    if let Err(e) = daemon.run().await {
        ui::print_status("✗", &format!("Daemon error: {}", e), ui::StatusType::Error);
        error!("Daemon error: {}", e);
        process::exit(1);
    }

    ui::print_status("✓", "JIOPad daemon stopped gracefully", ui::StatusType::Success);
    info!("JIOPad daemon stopped gracefully");
}

fn init_logging(args: &cli::Args) {
    use tracing_subscriber::{EnvFilter, fmt};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&args.log_level));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(true)
        .init();
}
