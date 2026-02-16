//! TOEditor - Table of Organization & Equipment Editor
//! 
//! A cross-platform tool for editing military organizational structures

use anyhow::Result;
use std::panic;

/// Custom panic handler
fn setup_panic_handler() {
    panic::set_hook(Box::new(|panic_info| {
        log::error!("Panic occurred:");
        if let Some(location) = panic_info.location() {
            log::error!("  Location: {}:{}:{}", location.file(), location.line(), location.column());
        }
        if let Some(message) = panic_info.payload().downcast_ref::<&str>() {
            log::error!("  Message: {}", message);
        } else if let Some(message) = panic_info.payload().downcast_ref::<String>() {
            log::error!("  Message: {}", message);
        }
    }));
}

fn main() -> Result<()> {
    env_logger::init();
    setup_panic_handler();
    
    // Initialize translations if needed
    // Note: Translations are embedded via build.rs, but we can initialize them here
    // For bundled translations, select_bundled_translation() should work
    
    // Initialize application
    let app = toeditor::app::AppMainWindow::new()?;
    
    // Run the UI
    app.run()?;
    
    Ok(())
}
