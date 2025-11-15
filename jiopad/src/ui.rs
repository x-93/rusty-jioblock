//! User interface utilities for better console output

use std::fmt;
use std::time::Duration;

/// ANSI color codes for terminal output
pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    
    // Colors
    pub const BLACK: &str = "\x1b[30m";
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m";
    
    // Bright colors
    pub const BRIGHT_BLACK: &str = "\x1b[90m";
    pub const BRIGHT_RED: &str = "\x1b[91m";
    pub const BRIGHT_GREEN: &str = "\x1b[92m";
    pub const BRIGHT_YELLOW: &str = "\x1b[93m";
    pub const BRIGHT_BLUE: &str = "\x1b[94m";
    pub const BRIGHT_MAGENTA: &str = "\x1b[95m";
    pub const BRIGHT_CYAN: &str = "\x1b[96m";
    pub const BRIGHT_WHITE: &str = "\x1b[97m";
}

/// Print startup banner
pub fn print_banner(version: &str, network: &str) {
    println!();
    println!("{}╔══════════════════════════════════════════════════════════════╗{}", colors::BRIGHT_CYAN, colors::RESET);
    println!("{}║{}                                                              {}║{}", colors::BRIGHT_CYAN, colors::RESET, colors::BRIGHT_CYAN, colors::RESET);
    println!("{}║{}          {}JIO BLOCKCHAIN NODE - JIOPAD v{}{}          {}║{}", 
        colors::BRIGHT_CYAN, colors::RESET, colors::BOLD, version, colors::RESET, colors::BRIGHT_CYAN, colors::RESET);
    println!("{}║{}                                                              {}║{}", colors::BRIGHT_CYAN, colors::RESET, colors::BRIGHT_CYAN, colors::RESET);
    println!("{}║{}  Network: {}{:<50}{}  {}║{}", 
        colors::BRIGHT_CYAN, colors::RESET, colors::BRIGHT_GREEN, network, colors::RESET, colors::BRIGHT_CYAN, colors::RESET);
    println!("{}║{}                                                              {}║{}", colors::BRIGHT_CYAN, colors::RESET, colors::BRIGHT_CYAN, colors::RESET);
    println!("{}╚══════════════════════════════════════════════════════════════╝{}", colors::BRIGHT_CYAN, colors::RESET);
    println!();
}

/// Print status line with icon and color
pub fn print_status(icon: &str, message: &str, status: StatusType) {
    let color = match status {
        StatusType::Success => colors::BRIGHT_GREEN,
        StatusType::Info => colors::BRIGHT_CYAN,
        StatusType::Warning => colors::BRIGHT_YELLOW,
        StatusType::Error => colors::BRIGHT_RED,
        StatusType::Neutral => colors::RESET,
    };
    
    println!("{}[{}]{} {} {}", color, icon, colors::RESET, color, message);
}

/// Status types for colored output
#[derive(Debug, Clone, Copy)]
pub enum StatusType {
    Success,
    Info,
    Warning,
    Error,
    Neutral,
}

/// Print a section header
pub fn print_section(title: &str) {
    println!();
    println!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", colors::DIM, colors::RESET);
    println!("{}  {}{}", colors::BRIGHT_CYAN, colors::BOLD, title);
    println!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", colors::DIM, colors::RESET);
    println!();
}

/// Print key-value pair in a formatted way
pub fn print_kv(key: &str, value: &str) {
    println!("  {}{}:{}{} {}{}{}", 
        colors::BRIGHT_WHITE, key, colors::RESET, colors::DIM, 
        colors::BRIGHT_CYAN, value, colors::RESET);
}

/// Print configuration summary
pub fn print_config_summary(config: &crate::config::Config) {
    print_section("Configuration");
    
    print_kv("Network", &config.network.network_id);
    print_kv("Data Directory", config.storage.data_dir.to_str().unwrap_or("N/A"));
    let rpc_status = if config.rpc.enabled {
        format!("{}:{}", config.rpc.bind_address, config.rpc.port)
    } else {
        "Disabled".to_string()
    };
    print_kv("RPC Server", &rpc_status);
    print_kv("P2P Port", &config.p2p.port.to_string());
    print_kv("Mining", if config.mining.enabled {
        "Enabled"
    } else {
        "Disabled"
    });
    
    if config.mining.enabled {
        if let Some(addr) = &config.mining.mining_address {
            print_kv("Mining Address", addr);
        }
        print_kv("Mining Threads", &config.mining.num_threads.to_string());
    }
}

/// Print component status
pub fn print_component_status(component: &str, status: ComponentStatus) {
    let (icon, color, text) = match status {
        ComponentStatus::Starting => ("⏳", colors::BRIGHT_YELLOW, "Starting"),
        ComponentStatus::Running => ("✓", colors::BRIGHT_GREEN, "Running"),
        ComponentStatus::Stopped => ("✗", colors::BRIGHT_RED, "Stopped"),
        ComponentStatus::Error => ("⚠", colors::BRIGHT_RED, "Error"),
    };
    
    println!("  {}[{}]{} {:<20} {}", color, icon, colors::RESET, component, text);
}

#[derive(Debug, Clone, Copy)]
pub enum ComponentStatus {
    Starting,
    Running,
    Stopped,
    Error,
}

/// Format duration as human-readable string
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;
        format!("{}h {}m {}s", hours, minutes, seconds)
    }
}

/// Format bytes as human-readable string
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    
    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[unit_idx])
    } else {
        format!("{:.2} {}", size, UNITS[unit_idx])
    }
}

/// Format hashrate as human-readable string
pub fn format_hashrate(hashrate: f64) -> String {
    if hashrate >= 1e12 {
        format!("{:.2} TH/s", hashrate / 1e12)
    } else if hashrate >= 1e9 {
        format!("{:.2} GH/s", hashrate / 1e9)
    } else if hashrate >= 1e6 {
        format!("{:.2} MH/s", hashrate / 1e6)
    } else if hashrate >= 1e3 {
        format!("{:.2} KH/s", hashrate / 1e3)
    } else {
        format!("{:.2} H/s", hashrate)
    }
}

/// Node status summary
pub struct NodeStatus {
    pub uptime: Duration,
    pub block_count: u64,
    pub peer_count: usize,
    pub is_mining: bool,
    pub mining_hashrate: f64,
    pub mempool_size: usize,
    pub sync_percentage: f64,
}

impl fmt::Display for NodeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", colors::DIM, colors::RESET)?;
        writeln!(f, "{}  Node Status Summary{}", colors::BRIGHT_CYAN, colors::RESET)?;
        writeln!(f, "{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", colors::DIM, colors::RESET)?;
        writeln!(f)?;
        writeln!(f, "  {}Uptime:{}           {}", colors::BRIGHT_WHITE, colors::RESET, format_duration(self.uptime))?;
        writeln!(f, "  {}Blocks:{}           {}", colors::BRIGHT_WHITE, colors::RESET, self.block_count)?;
        writeln!(f, "  {}Peers:{}            {}", colors::BRIGHT_WHITE, colors::RESET, self.peer_count)?;
        writeln!(f, "  {}Mempool:{}          {} transactions", colors::BRIGHT_WHITE, colors::RESET, self.mempool_size)?;
        writeln!(f, "  {}Sync:{}             {:.1}%", colors::BRIGHT_WHITE, colors::RESET, self.sync_percentage)?;
        
        if self.is_mining {
            writeln!(f, "  {}Mining:{}           {} {}", 
                colors::BRIGHT_WHITE, colors::RESET, 
                colors::BRIGHT_GREEN, format_hashrate(self.mining_hashrate))?;
        } else {
            writeln!(f, "  {}Mining:{}           {}Disabled{}", 
                colors::BRIGHT_WHITE, colors::RESET, 
                colors::DIM, colors::RESET)?;
        }
        
        writeln!(f)
    }
}

/// Progress bar for long operations
pub struct ProgressBar {
    width: usize,
    current: usize,
    total: usize,
    label: String,
}

impl ProgressBar {
    pub fn new(label: String, total: usize) -> Self {
        Self {
            width: 50,
            current: 0,
            total,
            label,
        }
    }
    
    pub fn update(&mut self, current: usize) {
        self.current = current.min(self.total);
        self.render();
    }
    
    pub fn increment(&mut self) {
        self.update(self.current + 1);
    }
    
    fn render(&self) {
        let percentage = if self.total > 0 {
            (self.current as f64 / self.total as f64 * 100.0) as usize
        } else {
            0
        };
        
        let filled = (self.width as f64 * percentage as f64 / 100.0) as usize;
        let empty = self.width - filled;
        
        print!("\r{}  {}: [", colors::BRIGHT_CYAN, self.label);
        print!("{}{}", colors::BRIGHT_GREEN, "█".repeat(filled));
        print!("{}{}", colors::DIM, "░".repeat(empty));
        print!("{}] {}% ({}/{}){}", colors::RESET, percentage, self.current, self.total, colors::RESET);
        
        use std::io::{self, Write};
        let _ = io::stdout().flush();
    }
    
    pub fn finish(&self) {
        println!();
    }
}

/// Check if terminal supports colors
pub fn supports_colors() -> bool {
    // On Windows, check if we're in a terminal that supports ANSI
    #[cfg(windows)]
    {
        use std::env;
        env::var("TERM").is_ok() || env::var("WT_SESSION").is_ok()
    }
    
    #[cfg(not(windows))]
    {
        use std::env;
        env::var("TERM").is_ok()
    }
}

