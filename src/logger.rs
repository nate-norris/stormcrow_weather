//! Starts async file-based log service for a headless program.
//! 
//! The global simplelog WriteLogger writes log messages to a "stormcrow.log" 
//! file co-located with the running executable or at a user specified location.

use std::env;
use std::path::{PathBuf, Path};
use std::fs::OpenOptions;
use simplelog::*;
use log::LevelFilter;

/// Starts async file-based log service for a headless program.
/// The file is created automatically if missing. 
/// 
/// # Usage
/// This should be called once at program startup, typically from `main()`:
///
/// ```no_run
/// logger::init_logger();
/// logger::info("Application started");
/// ```
pub fn init_logger(dir_path: Option<&Path>) {
    
    // determine director for log file
    let dir: PathBuf = match dir_path {
        Some(p) => p.to_path_buf(),
        None => env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(PathBuf::from))
            .unwrap_or_else(|| env::current_dir().unwrap()),
    };

    // open and create the file if needed
    let log_file = dir.join("stormcrow.log");
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)
        .unwrap();

    simplelog::WriteLogger::init(
        LevelFilter::Info,
        ConfigBuilder::new()
            .set_time_format_rfc3339()
            .set_location_level(LevelFilter::Error)
            .build(),
        log_file
    ).unwrap();
}

/// Writes information logs
/// Arguments must implement ['std::fmt:;Display']
/// 
/// # Arguments
/// * `message` — The information to log
pub fn info(message: impl std::fmt::Display) {
    log::info!("{}", message);
}

/// Writes error logs
/// Arguments must implement ['std::fmt:;Display']
/// 
/// # Arguments
/// `message` - the primary error text
/// `extra` - an additional value to append to log
/// 
/// # Examples
/// ```
/// logger::error("sensor failure", None);
/// logger::error("sensor failure:", Some(err));
/// ```
pub fn error(message: impl std::fmt::Display, extra: Option<impl std::fmt::Display>) {
    match extra {
        Some(e) => log::error!("{} {}", message, e),
        None => log::error!("{}", message),
    }
}