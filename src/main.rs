//! TOEditor - Table of Organization & Equipment Editor
//! 
//! A cross-platform tool for editing military organizational structures

mod app;
mod models;
mod db;
mod config;
mod export;
mod import;

use anyhow::Result;
use std::panic;

/// Custom panic handler
fn setup_panic_handler() {
    panic::set_hook(Box::new(|panic_info| {
        eprintln!("Panic occurred:");
        if let Some(location) = panic_info.location() {
            eprintln!("  Location: {}:{}:{}", location.file(), location.line(), location.column());
        }
        if let Some(message) = panic_info.payload().downcast_ref::<&str>() {
            eprintln!("  Message: {}", message);
        } else if let Some(message) = panic_info.payload().downcast_ref::<String>() {
            eprintln!("  Message: {}", message);
        }
    }));
}

fn main() -> Result<()> {
    setup_panic_handler();
    
    // Initialize application
    let app = app::AppMainWindow::new()?;
    
    // Run the UI
    app.run()?;
    
    Ok(())
}
