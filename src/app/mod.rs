//! Main application module

mod translations;
mod dialogs;
mod editors;

slint::include_modules!();

use anyhow::Result;
use slint::{ComponentHandle, Model, ModelRc, VecModel, Weak, SharedString};
use crate::i18n::Language;
use crate::models::{Library, validate_library};
use crate::services::LibraryService;
use crate::export;
use crate::db::Database;
use std::rc::Rc;
use std::cell::RefCell;

use translations::{ui_tr, apply_ui_translations};
use dialogs::{show_library_dialog, show_library_dialog_for_edit, show_error_dialog};
use editors::{show_branches_editor, show_branch_categories_editor, show_formation_levels_editor};

/// Application state shared between callbacks
pub(crate) struct AppState {
    pub(crate) database: Option<Database>,
    pub(crate) current_library: Option<Library>,
}

/// Main application window structure
pub struct AppMainWindow {
    window: MainWindow,
    #[allow(dead_code)]
    state: Rc<RefCell<AppState>>,
}

impl AppMainWindow {
    /// Create new main window
    pub fn new() -> Result<Self> {
        let window = MainWindow::new()?;
        let settings = crate::config::Settings::load().unwrap_or_default();

        // Load language from settings
        let lang = Language::from_code(&settings.language);

        // Try to set initial translation
        let lang_code = lang.code();
        if let Err(e) = slint::select_bundled_translation(lang_code) {
            log::warn!("Could not set initial translation: {}", e);
        }

        // Initialize database
        let db_path = settings.database_path.clone()
            .unwrap_or_else(|| crate::config::Settings::default_database_path().unwrap_or_default());
        let database = match crate::db::Database::open(&db_path) {
            Ok(db) => {
                log::info!("Database opened: {:?}", db_path);
                Some(db)
            }
            Err(e) => {
                log::warn!("Failed to open database: {}", e);
                None
            }
        };

        let state = Rc::new(RefCell::new(AppState {
            database,
            current_library: None,
        }));

        // Set initial theme from settings
        let theme = if settings.color_scheme == "dark" { "dark" } else { "light" };
        window.set_theme(theme.into());
        AppTheme::get(&window).set_mode(theme.into());
        log::info!("Initial theme set to: {}", theme);

        // Set up UI callbacks
        log::info!("Setting up callbacks...");
        setup_callbacks_with_state(&window, state.clone())?;
        log::info!("Callbacks set up successfully");

        // Initialize toolbar
        init_toolbar(&window)?;

        // Set window title
        let version = env!("CARGO_PKG_VERSION");
        window.set_window_title(format!("TOEditor v{}", version).into());

        // Set initial language property and UI strings from Rust
        window.set_current_language(lang_code.into());
        apply_ui_translations(&window, lang_code);
        log::info!("Initial language set to: {}", lang_code);

        // Load libraries into UI
        refresh_libraries_list(&window, state.clone());

        Ok(Self {
            window,
            state,
        })
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

/// Set up all UI callbacks with application state
fn setup_callbacks_with_state(
    window: &MainWindow,
    state: Rc<RefCell<AppState>>,
) -> Result<()> {
    let weak_window = window.as_weak();

    // Tabs and formations models
    let open_tabs_model = Rc::new(VecModel::from(vec![]));
    window.set_open_tabs(ModelRc::new(open_tabs_model.clone()));
    window.set_formations(ModelRc::new(VecModel::from(vec![])));

    // Language switching callback - MAIN callback with parameter
    // IMPORTANT: This must be registered BEFORE the window is shown
    window.on_switch_language({
        let weak = weak_window.clone();
        move |lang_code: SharedString| {
            log::debug!("Language switch to: {}", lang_code);

            if let Some(window) = weak.upgrade() {
                let lang = Language::from_code(lang_code.as_ref());

                // Update settings
                let mut settings = crate::config::Settings::load().unwrap_or_default();
                settings.language = lang.code().to_string();
                if let Err(e) = settings.save() {
                    log::error!("Failed to save settings: {}", e);
                }

                // Update the language property
                window.set_current_language(lang_code.clone());

                // Try to switch bundled translation
                if let Err(e) = slint::select_bundled_translation(lang_code.as_ref()) {
                    log::warn!("Translation API: {}", e);
                }

                // Update title and all UI strings
                let version = env!("CARGO_PKG_VERSION");
                let new_title = format!("TOEditor v{} [{}]", version, lang.name());
                window.set_window_title(new_title.into());
                apply_ui_translations(&window, lang_code.as_ref());
                window.window().request_redraw();
            }
        }
    });

    // Per-language callbacks delegate to the main switch_language callback
    window.on_switch_to_english({
        let weak = weak_window.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                window.invoke_switch_language("en".into());
            }
        }
    });

    window.on_switch_to_russian({
        let weak = weak_window.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                window.invoke_switch_language("ru".into());
            }
        }
    });

    // File menu actions
    window.on_file_exit({
        let weak = weak_window.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                log::debug!("File > Exit called");
                let _ = window.hide();
            }
        }
    });

    // Library management handlers
    let weak_window = window.as_weak();
    window.on_file_new_library(move || {
        log::debug!("File > New Library");
        if let Some(window) = weak_window.upgrade() {
            show_library_dialog(&window, "new", -1);
        }
    });

    // Library dialog handlers
    let state_clone = state.clone();
    let weak_window = window.as_weak();
    window.on_library_dialog_accepted(move |name: SharedString, country: SharedString, era: SharedString, author: SharedString, tags: SharedString, library_id: i32| {
        log::debug!("Library dialog accepted: name={}, country={}, era={}, author={}, tags={}, id={}",
                  name, country, era, author, tags, library_id);

        // Validate input
        let validation_errors = validate_library(name.as_str(), country.as_str(), era.as_str());
        if !validation_errors.is_empty() {
            let msg = validation_errors.iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            show_error_dialog("Validation Error", &msg);
            return;
        }

        // Parse tags
        let tags_vec: Vec<String> = tags.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let lib_to_update = {
            let state = state_clone.borrow();
            state.current_library.clone()
        };

        {
            let state = state_clone.borrow();
            if let Some(ref db) = state.database {
                let service = LibraryService::new(db.conn());

                if library_id == -1 {
                    // Create new library
                    let library = Library {
                        id: None,
                        name: name.to_string(),
                        country: country.to_string(),
                        era: era.to_string(),
                        author: author.to_string(),
                        version: 1,
                        tags: tags_vec,
                        units: Vec::new(),
                    };
                    match service.create_library(library) {
                        Ok(lib) => {
                            log::info!("Library created: {} (ID: {:?})", lib.name, lib.id);
                            drop(state);
                            let lib_id = lib.id.map(|x| x as i32).unwrap_or(-1);
                            state_clone.borrow_mut().current_library = Some(lib.clone());
                            if let Some(window) = weak_window.upgrade() {
                                window.set_current_library_name(lib.name.clone().into());
                                window.set_current_library_id(lib_id);
                                refresh_libraries_list(&window, state_clone.clone());
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to create library: {}", e);
                            show_error_dialog("Error", &format!("Failed to create library: {}", e));
                        }
                    }
                } else {
                    // Update existing library
                    if let Some(mut lib) = lib_to_update {
                        lib.name = name.to_string();
                        lib.country = country.to_string();
                        lib.era = era.to_string();
                        lib.author = author.to_string();
                        lib.tags = tags_vec.clone();
                        match service.save_library(lib.clone(), false) {
                            Ok(_) => {
                                log::info!("Library updated successfully");
                                drop(state);
                                let lib_id = lib.id.map(|x| x as i32).unwrap_or(-1);
                                state_clone.borrow_mut().current_library = Some(lib);
                                if let Some(window) = weak_window.upgrade() {
                                    window.set_current_library_name(name.clone());
                                    window.set_current_library_id(lib_id);
                                    refresh_libraries_list(&window, state_clone.clone());
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to update library: {}", e);
                                show_error_dialog("Error", &format!("Failed to update library: {}", e));
                            }
                        }
                    }
                }
            } else {
                log::error!("Database not initialized");
            }
        }
    });

    window.on_library_dialog_cancelled(|| {
        log::debug!("Library dialog cancelled");
    });

    let state_clone = state.clone();
    let weak_window = window.as_weak();
    window.on_file_open_library(move || {
        log::debug!("File > Open Library");
        // Refresh libraries list
        if let Some(window) = weak_window.upgrade() {
            refresh_libraries_list(&window, state_clone.clone());
        }
        // TODO: Show dialog to select library
    });

    // Sidebar toggles
    let weak_window = window.as_weak();
    window.on_toggle_libraries_sidebar(move || {
        if let Some(w) = weak_window.upgrade() {
            w.set_libraries_sidebar_expanded(!w.get_libraries_sidebar_expanded());
        }
    });
    let weak_window = window.as_weak();
    window.on_toggle_formations_sidebar(move || {
        if let Some(w) = weak_window.upgrade() {
            w.set_formations_sidebar_expanded(!w.get_formations_sidebar_expanded());
        }
    });

    // Library context menu (right-click): show LibraryContextMenu window
    let state_clone = state.clone();
    let weak_window = window.as_weak();
    window.on_library_right_clicked(move |library_id, item_index| {
        let lib_id = library_id;
        let menu = match LibraryContextMenu::new() {
            Ok(m) => m,
            Err(_) => return,
        };
        menu.set_library_id(lib_id);

        // Apply theme and translations to context menu
        if let Some(w) = weak_window.upgrade() {
            let lang = w.get_current_language().to_string();
            let theme = w.get_theme().to_string();
            AppTheme::get(&menu).set_mode(theme.into());
            menu.set_tr_properties(ui_tr(&lang, "Library Properties…").into());
            menu.set_tr_export(ui_tr(&lang, "Export Library…").into());
            menu.set_tr_history(ui_tr(&lang, "View History…").into());
            menu.set_tr_delete(ui_tr(&lang, "Delete").into());
        }

        // Position menu near the clicked library item
        let menu_window = menu.window();
        if let Some(main_window) = weak_window.upgrade() {
            let main_window_handle = main_window.window();
            let main_pos = main_window_handle.position();
            let item_height = 28;
            let sidebar_header_height = 50;
            let menu_x = (main_pos.x + 220) as f32;
            let menu_y = (main_pos.y + sidebar_header_height + (item_index * item_height)) as f32;
            menu_window.set_position(slint::WindowPosition::Logical(
                slint::LogicalPosition::new(menu_x, menu_y),
            ));
        }

        let weak_menu1 = menu.as_weak();
        let weak_menu2 = weak_menu1.clone();
        let weak_menu3 = weak_menu1.clone();
        let weak_menu4 = weak_menu1.clone();
        let state_c1 = state_clone.clone();
        let state_c2 = state_clone.clone();
        let state_c4 = state_clone.clone();
        let weak_win1 = weak_window.clone();
        let weak_win2 = weak_window.clone();
        let weak_win4 = weak_window.clone();
        menu.on_properties(move || {
            if let Some(m) = weak_menu1.upgrade() {
                m.hide().ok();
            }
            if let Some(w) = weak_win1.upgrade() {
                select_library_if_needed(state_c1.clone(), &w, lib_id);
                show_library_dialog_for_edit(&w, lib_id, state_c1.clone());
            }
        });
        menu.on_export_library(move || {
            if let Some(m) = weak_menu2.upgrade() {
                m.hide().ok();
            }
            if let Some(w) = weak_win2.upgrade() {
                select_library_if_needed(state_c2.clone(), &w, lib_id);
                w.invoke_file_export_library();
            }
        });
        menu.on_history(move || {
            if let Some(m) = weak_menu3.upgrade() {
                m.hide().ok();
            }
            log::debug!("Library > View history");
        });
        menu.on_delete_library(move || {
            if let Some(m) = weak_menu4.upgrade() {
                m.hide().ok();
            }
            if let Some(w) = weak_win4.upgrade() {
                select_library_if_needed(state_c4.clone(), &w, lib_id);
                w.invoke_library_delete();
            }
        });

        menu.show().ok();
    });

    // Library selection callback
    let state_clone = state.clone();
    let weak_window = window.as_weak();
    window.on_library_selected(move |library_id| {
        log::debug!("Library selected: {}", library_id);
        let has_db = {
            let state = state_clone.borrow();
            state.database.is_some()
        };

        if !has_db {
            log::error!("Database not initialized");
            return;
        }

        {
            let state = state_clone.borrow();
            if let Some(ref db) = state.database {
                let service = LibraryService::new(db.conn());
                match service.get_library(library_id as i64) {
                    Ok(Some(lib)) => {
                        log::info!("Loaded library: {}", lib.name);
                        drop(state);
                        state_clone.borrow_mut().current_library = Some(lib.clone());
                        if let Some(window) = weak_window.upgrade() {
                            window.set_current_library_name(lib.name.clone().into());
                            window.set_current_library_id(library_id);
                            refresh_formations_list(&window);
                        }
                    }
                    Err(e) => log::error!("Failed to load library: {}", e),
                    _ => {}
                }
            }
        }
    });

    // Formation open (add tab)
    let tabs1 = open_tabs_model.clone();
    let weak_win_tabs = window.as_weak();
    window.on_formation_open(move |formation_id| {
        let title = format!("Formation {}", formation_id);
        let tab = FormationTab {
            id: formation_id,
            title: title.clone().into(),
            view_mode: "table".into(),
        };
        tabs1.push(tab);
        let idx = tabs1.row_count() - 1;
        if let Some(w) = weak_win_tabs.upgrade() {
            w.set_current_tab_index(idx as i32);
            w.set_current_tab_title(title.into());
            w.set_current_tab_view_mode("table".into());
        }
    });

    // Tab select / close / set view mode
    let tabs2 = open_tabs_model.clone();
    let weak_win_tabs2 = window.as_weak();
    window.on_tab_select(move |index| {
        if let Some(w) = weak_win_tabs2.upgrade() {
            w.set_current_tab_index(index);
            if index >= 0 && (index as usize) < tabs2.row_count() {
                if let Some(row) = tabs2.row_data(index as usize) {
                    w.set_current_tab_title(row.title.clone());
                    w.set_current_tab_view_mode(row.view_mode.clone());
                }
            }
        }
    });
    let tabs3 = open_tabs_model.clone();
    let weak_win_tabs3 = window.as_weak();
    window.on_tab_close(move |index| {
        if index >= 0 && (index as usize) < tabs3.row_count() {
            tabs3.remove(index as usize);
            if let Some(w) = weak_win_tabs3.upgrade() {
                let count = tabs3.row_count();
                if count == 0 {
                    w.set_current_tab_index(-1);
                    w.set_current_tab_title("".into());
                    w.set_current_tab_view_mode("table".into());
                } else {
                    let new_idx = (index as usize).min(count.saturating_sub(1));
                    w.set_current_tab_index(new_idx as i32);
                    if let Some(row) = tabs3.row_data(new_idx) {
                        w.set_current_tab_title(row.title.clone());
                        w.set_current_tab_view_mode(row.view_mode.clone());
                    }
                }
            }
        }
    });
    let tabs4 = open_tabs_model.clone();
    let weak_win_tabs4 = window.as_weak();
    window.on_tab_set_view_mode(move |index, mode| {
        if index >= 0 && (index as usize) < tabs4.row_count() {
            if let Some(mut row) = tabs4.row_data(index as usize) {
                row.view_mode = mode.clone();
                tabs4.set_row_data(index as usize, row);
            }
            if let Some(w) = weak_win_tabs4.upgrade() {
                if w.get_current_tab_index() == index {
                    w.set_current_tab_view_mode(mode);
                }
            }
        }
    });

    window.on_file_recent_libraries(|| {
        log::debug!("File > Recent Libraries");
    });

    let state_clone = state.clone();
    window.on_file_save_library(move || {
        log::debug!("File > Save Library");
        let lib_to_save = {
            let state = state_clone.borrow();
            state.current_library.clone()
        };

        if let Some(lib) = lib_to_save {
            let state = state_clone.borrow();
            if let Some(ref db) = state.database {
                let service = LibraryService::new(db.conn());
                match service.save_library(lib, true) {
                    Ok(_) => {
                        drop(state);
                        log::info!("Library saved successfully");
                    }
                    Err(e) => {
                        drop(state);
                        log::error!("Failed to save library: {}", e);
                        show_error_dialog("Error", &format!("Failed to save library: {}", e));
                    }
                }
            } else {
                log::error!("Database not initialized");
            }
        } else {
            log::warn!("No library to save. Create or open a library first.");
        }
    });

    window.on_file_save_library_as(|| {
        log::debug!("File > Save Library As");
        show_error_dialog("Not implemented", "Save Library As is not yet implemented.");
    });

    let state_clone = state.clone();
    window.on_file_import_library(move || {
        log::debug!("File > Import Library");
        let state = state_clone.borrow();
        if state.database.is_some() {
            log::info!("Import functionality - TODO: implement file dialog");
        }
    });

    window.on_file_import_formation(|| {
        log::debug!("File > Import Formation");
        show_error_dialog("Not implemented", "Import Formation is not yet implemented.");
    });

    let state_clone = state.clone();
    window.on_file_export_library(move || {
        log::debug!("File > Export Library");
        let state = state_clone.borrow();
        if let Some(ref lib) = state.current_library {
            let path = std::env::temp_dir().join(format!("{}.json", lib.name));
            match export::export_json(lib, &path) {
                Ok(_) => log::info!("Library exported to: {:?}", path),
                Err(e) => {
                    log::error!("Failed to export library: {}", e);
                    show_error_dialog("Export Error", &format!("Failed to export library: {}", e));
                }
            }
        } else {
            log::warn!("No library to export. Create or open a library first.");
        }
    });

    window.on_file_export_formation(|| {
        log::debug!("File > Export Formation");
        show_error_dialog("Not implemented", "Export Formation is not yet implemented.");
    });
    window.on_file_export_spreadsheet(|| {
        log::debug!("File > Export Spreadsheet");
        show_error_dialog("Not implemented", "Export Spreadsheet is not yet implemented.");
    });
    window.on_file_export_diagram(|| {
        log::debug!("File > Export Diagram");
        show_error_dialog("Not implemented", "Export Diagram is not yet implemented.");
    });

    // Edit menu actions
    window.on_edit_find(|| { log::debug!("Edit > Find"); show_error_dialog("Not implemented", "Find is not yet implemented."); });
    window.on_edit_find_replace(|| { log::debug!("Edit > Find and Replace"); show_error_dialog("Not implemented", "Find and Replace is not yet implemented."); });
    window.on_edit_undo(|| { log::debug!("Edit > Undo"); show_error_dialog("Not implemented", "Undo is not yet implemented."); });
    window.on_edit_redo(|| { log::debug!("Edit > Redo"); show_error_dialog("Not implemented", "Redo is not yet implemented."); });
    window.on_edit_cut(|| { log::debug!("Edit > Cut"); show_error_dialog("Not implemented", "Cut is not yet implemented."); });
    window.on_edit_copy(|| { log::debug!("Edit > Copy"); show_error_dialog("Not implemented", "Copy is not yet implemented."); });
    window.on_edit_paste(|| { log::debug!("Edit > Paste"); show_error_dialog("Not implemented", "Paste is not yet implemented."); });
    window.on_edit_delete(|| { log::debug!("Edit > Delete"); show_error_dialog("Not implemented", "Delete is not yet implemented."); });
    window.on_edit_add_formation(|| { log::debug!("Edit > Add Formation"); show_error_dialog("Not implemented", "Add Formation is not yet implemented."); });
    window.on_edit_edit_properties(|| { log::debug!("Edit > Edit Properties"); show_error_dialog("Not implemented", "Edit Properties is not yet implemented."); });

    // Library menu actions
    window.on_library_positions_editor(|| { log::debug!("Library > Positions Editor"); show_error_dialog("Not implemented", "Positions and Ranks Editor is not yet implemented."); });
    window.on_library_equipment_editor(|| { log::debug!("Library > Equipment Editor"); show_error_dialog("Not implemented", "Equipment and Vehicles Editor is not yet implemented."); });

    // Formation levels editor (separate window)
    let state_formation = state.clone();
    let weak_window_formation = window.as_weak();
    window.on_library_formation_levels(move || {
        let (lib_id, lib_name) = {
            let st = state_formation.borrow();
            match &st.current_library {
                Some(lib) => match lib.id {
                    Some(id) => (id, lib.name.clone()),
                    None => {
                        log::warn!("Library has no id");
                        return;
                    }
                },
                None => {
                    log::warn!("No library selected");
                    return;
                }
            }
        };
        let lang = weak_window_formation
            .upgrade()
            .map(|w| w.get_current_language().to_string())
            .unwrap_or_else(|| "en".to_string());
        show_formation_levels_editor(state_formation.clone(), lib_id, &lib_name, &lang);
    });

    // Branches editor (separate window)
    let state_branches = state.clone();
    let weak_win_branches = window.as_weak();
    window.on_library_branches(move || {
        let (lib_id, lib_name) = {
            let st = state_branches.borrow();
            match &st.current_library {
                Some(lib) => match lib.id {
                    Some(id) => (id, lib.name.clone()),
                    None => {
                        log::warn!("Library has no id");
                        return;
                    }
                },
                None => {
                    log::warn!("No library selected");
                    return;
                }
            }
        };
        let lang = weak_win_branches
            .upgrade()
            .map(|w| w.get_current_language().to_string())
            .unwrap_or_else(|| "en".to_string());
        show_branches_editor(state_branches.clone(), lib_id, &lib_name, &lang);
    });

    // Branch categories editor (separate window)
    let state_cat = state.clone();
    let weak_win_cat = window.as_weak();
    window.on_library_branch_categories(move || {
        let (lib_id, lib_name) = {
            let st = state_cat.borrow();
            match &st.current_library {
                Some(lib) => match lib.id {
                    Some(id) => (id, lib.name.clone()),
                    None => {
                        log::warn!("Library has no id");
                        return;
                    }
                },
                None => {
                    log::warn!("No library selected");
                    return;
                }
            }
        };
        let lang = weak_win_cat
            .upgrade()
            .map(|w| w.get_current_language().to_string())
            .unwrap_or_else(|| "en".to_string());
        show_branch_categories_editor(state_cat.clone(), lib_id, &lib_name, &lang);
    });

    // Library properties/edit - show dialog
    let state_clone = state.clone();
    let weak_window = window.as_weak();
    window.on_library_properties(move || {
        log::debug!("Library > Properties");
        let state = state_clone.borrow();
        if let Some(ref lib) = state.current_library {
                if let Some(lib_id) = lib.id {
                drop(state);
                if let Some(window) = weak_window.upgrade() {
                    show_library_dialog_for_edit(&window, lib_id as i32, state_clone.clone());
                }
            }
        } else {
            log::warn!("No library selected");
        }
    });

    window.on_library_manage_tags(|| { log::debug!("Library > Manage Tags"); show_error_dialog("Not implemented", "Manage Tags is not yet implemented."); });
    window.on_library_export_library(|| { log::debug!("Library > Export Library"); show_error_dialog("Not implemented", "Export Library is not yet implemented."); });
    window.on_library_view_history(|| { log::debug!("Library > View History"); show_error_dialog("Not implemented", "View History is not yet implemented."); });
    window.on_library_create_snapshot(|| { log::debug!("Library > Create Snapshot"); show_error_dialog("Not implemented", "Create Snapshot is not yet implemented."); });
    window.on_library_compare_versions(|| { log::debug!("Library > Compare Versions"); show_error_dialog("Not implemented", "Compare Versions is not yet implemented."); });
    window.on_library_revert_to_version(|| { log::debug!("Library > Revert to Version"); show_error_dialog("Not implemented", "Revert to Version is not yet implemented."); });

    // Library delete: show confirmation dialog, then delete on confirm
    let state_clone = state.clone();
    let weak_window = window.as_weak();
    window.on_library_delete(move || {
        log::debug!("Library > Delete");
        let (lib_id, lib_name) = {
            let state = state_clone.borrow();
            match &state.current_library {
                Some(lib) => match lib.id {
                    Some(id) => (id, lib.name.clone()),
                    None => {
                        log::warn!("Library has no id");
                        return;
                    }
                },
                None => {
                    log::warn!("No library selected");
                    return;
                }
            }
        };
        let lang = weak_window
            .upgrade()
            .map(|w| w.get_current_language().to_string())
            .unwrap_or_else(|| "en".to_string());
        let message = ui_tr(&lang, "Delete library \"{}\"? This will delete all versions.").replace("{}", &lib_name);
        let dialog = match ConfirmDeleteDialog::new() {
            Ok(d) => d,
            Err(e) => {
                log::error!("Failed to create confirm dialog: {}", e);
                return;
            }
        };
        dialog.set_dialog_title(ui_tr(&lang, "Delete library?").into());
        dialog.set_cancel_text(ui_tr(&lang, "Cancel").into());
        dialog.set_delete_text(ui_tr(&lang, "Delete").into());
        dialog.set_message(message.into());
        let weak_dialog1 = dialog.as_weak();
        let weak_dialog2 = dialog.as_weak();
        let state_for_confirm = state_clone.clone();
        let weak_window_confirm = weak_window.clone();
        dialog.on_confirmed(move || {
            if let Some(d) = weak_dialog1.upgrade() {
                d.hide().unwrap_or_default();
            }
            let delete_ok = {
                let state = state_for_confirm.borrow();
                if let Some(ref db) = state.database {
                    let service = LibraryService::new(db.conn());
                    match service.delete_library(lib_id) {
                        Ok(_) => true,
                        Err(e) => {
                            log::error!("Failed to delete library: {}", e);
                            show_error_dialog("Error", &format!("Failed to delete library: {}", e));
                            false
                        }
                    }
                } else {
                    false
                }
            };
            if delete_ok {
                log::info!("Library deleted successfully");
                state_for_confirm.borrow_mut().current_library = None;
                if let Some(window) = weak_window_confirm.upgrade() {
                    window.set_current_library_name("".into());
                    window.set_current_library_id(-1);
                    refresh_libraries_list(&window, state_for_confirm.clone());
                }
            }
        });
        dialog.on_cancelled(move || {
            if let Some(d) = weak_dialog2.upgrade() {
                d.hide().unwrap_or_default();
            }
        });
        dialog.show().unwrap_or_default();
    });

    // Unit menu actions
    window.on_unit_add_child(|| { log::debug!("Unit > Add Child"); show_error_dialog("Not implemented", "Add Child Formation is not yet implemented."); });
    window.on_unit_delete(|| { log::debug!("Unit > Delete"); show_error_dialog("Not implemented", "Delete Formation is not yet implemented."); });
    window.on_unit_move_up(|| { log::debug!("Unit > Move Up"); show_error_dialog("Not implemented", "Move Up is not yet implemented."); });
    window.on_unit_move_down(|| { log::debug!("Unit > Move Down"); show_error_dialog("Not implemented", "Move Down is not yet implemented."); });
    window.on_unit_summary_table(|| { log::debug!("Unit > Summary Table"); show_error_dialog("Not implemented", "Summary Table is not yet implemented."); });
    window.on_unit_export(|| { log::debug!("Unit > Export"); show_error_dialog("Not implemented", "Export Formation is not yet implemented."); });
    window.on_unit_view_history(|| { log::debug!("Unit > View History"); show_error_dialog("Not implemented", "View History is not yet implemented."); });
    window.on_unit_create_snapshot(|| { log::debug!("Unit > Create Snapshot"); show_error_dialog("Not implemented", "Create Snapshot is not yet implemented."); });
    window.on_unit_compare_versions(|| { log::debug!("Unit > Compare Versions"); show_error_dialog("Not implemented", "Compare Versions is not yet implemented."); });
    window.on_unit_revert_to_version(|| { log::debug!("Unit > Revert to Version"); show_error_dialog("Not implemented", "Revert to Version is not yet implemented."); });

    // View menu actions
    window.on_view_table(|| { log::debug!("View > Table"); });
    window.on_view_diagram(|| { log::debug!("View > Diagram"); });
    window.on_view_table_and_diagram(|| { log::debug!("View > Table and Diagram"); });
    window.on_view_symbols_nato(|| { log::debug!("View > Symbols NATO"); show_error_dialog("Not implemented", "NATO symbols are not yet implemented."); });
    window.on_view_symbols_russia(|| { log::debug!("View > Symbols Russia"); show_error_dialog("Not implemented", "Russian symbols are not yet implemented."); });
    window.on_view_load_symbols(|| { log::debug!("View > Load Symbols"); show_error_dialog("Not implemented", "Load Custom Symbols is not yet implemented."); });
    // Theme switching callback
    let weak_window = window.as_weak();
    window.on_switch_theme(move |theme: slint::SharedString| {
        let theme_str = theme.to_string();
        log::debug!("Switching theme to: {}", theme_str);

        if let Some(w) = weak_window.upgrade() {
            w.set_theme(theme.clone());
            AppTheme::get(&w).set_mode(theme.clone());

            // Save theme to settings
            let mut settings = crate::config::Settings::load().unwrap_or_default();
            settings.color_scheme = theme_str.clone();
            if let Err(e) = settings.save() {
                log::error!("Failed to save theme setting: {}", e);
            } else {
                log::info!("Theme saved: {}", theme_str);
            }
        }
    });
    window.on_view_show_images(|| { log::debug!("View > Show Images"); show_error_dialog("Not implemented", "Show Equipment Images is not yet implemented."); });
    window.on_view_zoom_in(|| { log::debug!("View > Zoom In"); show_error_dialog("Not implemented", "Zoom In is not yet implemented."); });
    window.on_view_zoom_out(|| { log::debug!("View > Zoom Out"); show_error_dialog("Not implemented", "Zoom Out is not yet implemented."); });
    window.on_view_zoom_reset(|| { log::debug!("View > Zoom Reset"); show_error_dialog("Not implemented", "Reset Zoom is not yet implemented."); });
    window.on_view_refresh(|| { log::debug!("View > Refresh"); });

    // Tools menu actions
    window.on_tools_settings(|| { log::debug!("Tools > Settings"); show_error_dialog("Not implemented", "Settings dialog is not yet implemented."); });
    window.on_tools_language(|| { log::debug!("Tools > Language"); });
    window.on_tools_data_paths(|| { log::debug!("Tools > Data Paths"); show_error_dialog("Not implemented", "Data Paths dialog is not yet implemented."); });
    window.on_tools_reset_settings(|| { log::debug!("Tools > Reset Settings"); show_error_dialog("Not implemented", "Reset Settings is not yet implemented."); });

    // Help menu actions
    window.on_help_user_guide(|| { log::debug!("Help > User Guide"); show_error_dialog("Not implemented", "User Guide is not yet available."); });
    window.on_help_about(|| { log::debug!("Help > About"); show_error_dialog("About TOEditor", "TOEditor - Table of Organization Editor\nA desktop application for creating and managing military organizational structures."); });
    window.on_help_check_updates(|| { log::debug!("Help > Check Updates"); show_error_dialog("Not implemented", "Check for Updates is not yet implemented."); });

    Ok(())
}

/// Ensure the given library is loaded as current; select it in UI if needed.
fn select_library_if_needed(state: Rc<RefCell<AppState>>, window: &MainWindow, library_id: i32) {
    let need_load = {
        let st = state.borrow();
        st.current_library.as_ref().and_then(|l| l.id) != Some(library_id as i64)
    };
    if need_load {
        let lib_result = {
            let st = state.borrow();
            if let Some(ref db) = st.database {
                let svc = LibraryService::new(db.conn());
                svc.get_library(library_id as i64)
            } else {
                return;
            }
        };
        if let Ok(Some(lib)) = lib_result {
            state.borrow_mut().current_library = Some(lib.clone());
            window.set_current_library_name(lib.name.clone().into());
            window.set_current_library_id(library_id);
        }
    }
}

/// Refresh formations list in the UI (placeholder until we have real formation tree).
fn refresh_formations_list(window: &MainWindow) {
    let formations = vec![
        FormationTreeItem {
            id: 1,
            name: "Root formation".into(),
            depth: 0,
        },
    ];
    window.set_formations(ModelRc::new(VecModel::from(formations)));
}

/// Refresh libraries list in the UI
fn refresh_libraries_list(window: &MainWindow, state: Rc<RefCell<AppState>>) {
    let state = state.borrow();
    if let Some(ref db) = state.database {
        let service = LibraryService::new(db.conn());
        match service.list_libraries() {
            Ok(libraries) => {
                let library_items: Vec<LibraryItem> = libraries
                    .iter()
                    .filter_map(|lib| {
                        lib.id.map(|id| LibraryItem {
                            id: id as i32,
                            name: lib.name.clone().into(),
                            country: lib.country.clone().into(),
                            era: lib.era.clone().into(),
                        })
                    })
                    .collect();
                window.set_libraries(ModelRc::new(VecModel::from(library_items)));
                log::info!("Refreshed libraries list: {} libraries", libraries.len());
            }
            Err(e) => {
                log::error!("Failed to load libraries: {}", e);
            }
        }
    }
}

/// Initialize toolbar
fn init_toolbar(window: &MainWindow) -> Result<()> {
    let toolbar_buttons = vec![
        ToolbarButton {
            id: "new_library".into(),
            text: "📄 New".into(),
            icon: "icons/document-new.svg".into(),
            tooltip: "New Library (Ctrl+N)".into(),
            enabled: true,
            is_separator: false,
        },
        ToolbarButton {
            id: "open_library".into(),
            text: "📂 Open".into(),
            icon: "icons/document-open.svg".into(),
            tooltip: "Open Library (Ctrl+O)".into(),
            enabled: true,
            is_separator: false,
        },
        ToolbarButton {
            id: "save_library".into(),
            text: "💾 Save".into(),
            icon: "icons/document-save.svg".into(),
            tooltip: "Save Library (Ctrl+S)".into(),
            enabled: true,
            is_separator: false,
        },
        ToolbarButton {
            id: "".into(),
            text: "".into(),
            icon: "".into(),
            tooltip: "".into(),
            enabled: false,
            is_separator: true,
        },
        ToolbarButton {
            id: "new_formation".into(),
            text: "➕ Formation".into(),
            icon: "icons/list-add.svg".into(),
            tooltip: "New Formation".into(),
            enabled: false,
            is_separator: false,
        },
        ToolbarButton {
            id: "".into(),
            text: "".into(),
            icon: "".into(),
            tooltip: "".into(),
            enabled: false,
            is_separator: true,
        },
        ToolbarButton {
            id: "table_view".into(),
            text: "📊 Table".into(),
            icon: "icons/view-table.svg".into(),
            tooltip: "Table View".into(),
            enabled: true,
            is_separator: false,
        },
        ToolbarButton {
            id: "diagram_view".into(),
            text: "🔀 Diagram".into(),
            icon: "icons/view-diagram.svg".into(),
            tooltip: "Diagram View".into(),
            enabled: true,
            is_separator: false,
        },
    ];

    window.set_toolbar(ModelRc::new(VecModel::from(toolbar_buttons)));

    // Wire up toolbar button clicks to existing callbacks
    let weak = window.as_weak();
    window.on_toolbar_clicked(move |id: SharedString| {
        if let Some(w) = weak.upgrade() {
            match id.as_str() {
                "new_library" => w.invoke_file_new_library(),
                "open_library" => w.invoke_file_open_library(),
                "save_library" => w.invoke_file_save_library(),
                "new_formation" => w.invoke_edit_add_formation(),
                "table_view" => w.invoke_view_table(),
                "diagram_view" => w.invoke_view_diagram(),
                other => log::debug!("Unknown toolbar button: {}", other),
            }
        }
    });

    Ok(())
}
