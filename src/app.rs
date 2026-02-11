//! Main application module

slint::include_modules!();

use anyhow::Result;
use slint::{ComponentHandle, ModelRc, VecModel, Weak};

/// Main application window structure
pub struct AppMainWindow {
    window: MainWindow,
}

impl AppMainWindow {
    /// Create new main window
    pub fn new() -> Result<Self> {
        let window = MainWindow::new()?;
        
        // Set up UI callbacks
        setup_callbacks(&window)?;
        
        // Initialize toolbar
        init_toolbar(&window)?;
        
        // Set window title
        let version = env!("CARGO_PKG_VERSION");
        window.set_window_title(format!("TOEditor v{}", version).into());
        
        Ok(Self { window })
    }
    
    /// Run the application
    pub fn run(self) -> Result<(), slint::PlatformError> {
        self.window.run()
    }
    
    /// Get weak reference to window
    pub fn as_weak(&self) -> Weak<MainWindow> {
        self.window.as_weak()
    }
}

/// Set up all UI callbacks
fn setup_callbacks(window: &MainWindow) -> Result<()> {
    let weak_window = window.as_weak();
    
    // File menu actions
    window.on_new_library({
        let weak = weak_window.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                handle_new_library(&window);
            }
        }
    });
    
    window.on_open_library({
        let weak = weak_window.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                handle_open_library(&window);
            }
        }
    });
    
    window.on_save_library({
        let weak = weak_window.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                handle_save_library(&window);
            }
        }
    });
    
    // Edit menu actions
    window.on_new_formation({
        let weak = weak_window.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                handle_new_formation(&window);
            }
        }
    });
    
    // View menu actions
    window.on_toggle_sidebar({
        let weak = weak_window.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                handle_toggle_sidebar(&window);
            }
        }
    });
    
    // Help menu actions
    window.on_about_dialog({
        let weak = weak_window.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                handle_about_dialog(&window);
            }
        }
    });
    
    Ok(())
}

/// Initialize toolbar
fn init_toolbar(window: &MainWindow) -> Result<()> {
    let toolbar_buttons = vec![
        ToolbarButton {
            id: "new_library".into(),
            icon: "icons/document-new.svg".into(),
            tooltip: "New Library (Ctrl+N)".into(),
            enabled: true,
            is_separator: false,
        },
        ToolbarButton {
            id: "open_library".into(),
            icon: "icons/document-open.svg".into(),
            tooltip: "Open Library (Ctrl+O)".into(),
            enabled: true,
            is_separator: false,
        },
        ToolbarButton {
            id: "save_library".into(),
            icon: "icons/document-save.svg".into(),
            tooltip: "Save Library (Ctrl+S)".into(),
            enabled: false,
            is_separator: false,
        },
        ToolbarButton {
            id: "".into(),
            icon: "".into(),
            tooltip: "".into(),
            enabled: false,
            is_separator: true,
        },
        ToolbarButton {
            id: "new_formation".into(),
            icon: "icons/list-add.svg".into(),
            tooltip: "New Formation (Ctrl+F)".into(),
            enabled: false,
            is_separator: false,
        },
        ToolbarButton {
            id: "".into(),
            icon: "".into(),
            tooltip: "".into(),
            enabled: false,
            is_separator: true,
        },
        ToolbarButton {
            id: "table_view".into(),
            icon: "icons/view-table.svg".into(),
            tooltip: "Table View (F2)".into(),
            enabled: true,
            is_separator: false,
        },
        ToolbarButton {
            id: "diagram_view".into(),
            icon: "icons/view-diagram.svg".into(),
            tooltip: "Diagram View (F3)".into(),
            enabled: true,
            is_separator: false,
        },
    ];
    
    window.set_toolbar(ModelRc::new(VecModel::from(toolbar_buttons)));
    Ok(())
}

/// Handle new library action
fn handle_new_library(window: &MainWindow) {
    // TODO: Show dialog to create new library
    eprintln!("New library action triggered");
    window.invoke_show_new_library_dialog();
}

/// Handle open library action
fn handle_open_library(window: &MainWindow) {
    // TODO: Show file dialog to open library
    eprintln!("Open library action triggered");
    window.invoke_show_open_library_dialog();
}

/// Handle save library action
fn handle_save_library(window: &MainWindow) {
    // TODO: Save current library
    eprintln!("Save library action triggered");
    window.invoke_save_current_library();
}

/// Handle new formation action
fn handle_new_formation(window: &MainWindow) {
    // TODO: Show dialog to create new formation
    eprintln!("New formation action triggered");
    window.invoke_show_new_formation_dialog();
}

/// Handle toggle sidebar action
fn handle_toggle_sidebar(window: &MainWindow) {
    let current = window.get_sidebar_visible();
    window.set_sidebar_visible(!current);
}

/// Handle about dialog action
fn handle_about_dialog(window: &MainWindow) {
    // TODO: Show about dialog
    eprintln!("About dialog action triggered");
    window.invoke_show_about_dialog();
}
