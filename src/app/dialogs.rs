//! Library dialog functions

use std::rc::Rc;
use std::cell::RefCell;
use slint::ComponentHandle;

use super::{MainWindow, LibraryDialog, AppState};

/// Show library dialog for creating new library
pub(super) fn show_library_dialog(window: &MainWindow, _mode: &str, library_id: i32) {
    let dialog = match LibraryDialog::new() {
        Ok(d) => d,
        Err(e) => {
            log::error!("Failed to create library dialog: {}", e);
            return;
        }
    };

    // New library - set defaults
    dialog.set_library_name("".into());
    dialog.set_library_country("US".into());
    dialog.set_library_era("2024".into());
    dialog.set_library_author("".into());
    dialog.set_library_tags("".into());

    let weak_dialog1 = dialog.as_weak();
    let weak_dialog2 = weak_dialog1.clone();
    let weak_window1 = window.as_weak();
    let weak_window2 = window.as_weak();

    dialog.on_accepted(move || {
        if let Some(d) = weak_dialog1.upgrade() {
            let name = d.get_library_name();
            let country = d.get_library_country();
            let era = d.get_library_era();
            let author = d.get_library_author();
            let tags = d.get_library_tags();

            if let Some(w) = weak_window1.upgrade() {
                w.invoke_library_dialog_accepted(name, country, era, author, tags, library_id);
            }
            d.hide().unwrap_or_default();
        }
    });

    dialog.on_cancelled(move || {
        if let Some(d) = weak_dialog2.upgrade() {
            if let Some(w) = weak_window2.upgrade() {
                w.invoke_library_dialog_cancelled();
            }
            d.hide().unwrap_or_default();
        }
    });

    dialog.show().unwrap_or_default();
}

/// Show a simple error dialog with a message and an OK button.
#[allow(dead_code)]
pub(crate) fn show_error_dialog(title: &str, message: &str) {
    let dialog = match super::ErrorDialog::new() {
        Ok(d) => d,
        Err(e) => {
            log::error!("Failed to create error dialog: {} (message was: {})", e, message);
            return;
        }
    };
    dialog.set_dialog_title(title.into());
    dialog.set_message(message.into());

    let weak = dialog.as_weak();
    dialog.on_dismissed(move || {
        if let Some(d) = weak.upgrade() {
            d.hide().unwrap_or_default();
        }
    });

    dialog.show().unwrap_or_default();
}

/// Show library dialog for editing existing library
pub(super) fn show_library_dialog_for_edit(window: &MainWindow, library_id: i32, state: Rc<RefCell<AppState>>) {
    let dialog = match LibraryDialog::new() {
        Ok(d) => d,
        Err(e) => {
            log::error!("Failed to create library dialog: {}", e);
            return;
        }
    };

    // Load library data for editing
    let lib_data = {
        let state = state.borrow();
        state.current_library.clone()
    };

    if let Some(ref lib) = lib_data {
        dialog.set_library_name(lib.name.clone().into());
        dialog.set_library_country(lib.country.clone().into());
        dialog.set_library_era(lib.era.clone().into());
        dialog.set_library_author(lib.author.clone().into());
        dialog.set_library_tags(lib.tags.join(", ").into());
    }

    let weak_dialog1 = dialog.as_weak();
    let weak_dialog2 = weak_dialog1.clone();
    let weak_window1 = window.as_weak();
    let weak_window2 = window.as_weak();

    dialog.on_accepted(move || {
        if let Some(d) = weak_dialog1.upgrade() {
            let name = d.get_library_name();
            let country = d.get_library_country();
            let era = d.get_library_era();
            let author = d.get_library_author();
            let tags = d.get_library_tags();

            if let Some(w) = weak_window1.upgrade() {
                w.invoke_library_dialog_accepted(name, country, era, author, tags, library_id);
            }
            d.hide().unwrap_or_default();
        }
    });

    dialog.on_cancelled(move || {
        if let Some(d) = weak_dialog2.upgrade() {
            if let Some(w) = weak_window2.upgrade() {
                w.invoke_library_dialog_cancelled();
            }
            d.hide().unwrap_or_default();
        }
    });

    dialog.show().unwrap_or_default();
}
