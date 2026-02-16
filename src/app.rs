//! Main application module

slint::include_modules!();

use anyhow::Result;
use slint::{ComponentHandle, Model, ModelRc, VecModel, Weak, SharedString};
use crate::i18n::{Language, TranslationManager};
use crate::models::{Library, StandardFormationLevel, CustomFormationLevel, Branch};
use crate::services::LibraryService;
use crate::export::{self, export_branches_to_path, import_branches_from_path, export_branch_categories_to_path, import_branch_categories_from_path, export_formation_levels_to_path, import_formation_levels_from_path, copy_branches_between_libraries, copy_branch_categories_between_libraries, copy_formation_levels_between_libraries};
use crate::db::Database;
use crate::db::repositories::{FormationLevelRepo, BranchRepo, BranchCategoryRepo};
use crate::config::Settings;
use std::rc::Rc;
use std::cell::RefCell;

/// Return translated string for UI (toolbar, sidebar, welcome, menu). Keys match msgid from .po.
fn ui_tr(lang: &str, key: &str) -> String {
    if lang != "ru" {
        return key.to_string();
    }
    let s = match key {
        "New Library" => "Новая библиотека",
        "Open Library" => "Открыть библиотеку",
        "Open Library…" => "Открыть библиотеку…",
        "Save Library" => "Сохранить библиотеку",
        "Language" => "Язык",
        "Libraries" => "Библиотеки",
        "Welcome to TOEditor" => "Добро пожаловать в TOEditor",
        "Create a new library or open an existing one to get started." => {
            "Создайте новую библиотеку или откройте существующую, чтобы начать работу."
        }
        "File" => "Файл",
        "Recent Libraries" => "Последние библиотеки",
        "Save Library As…" => "Сохранить библиотеку как…",
        "Import" => "Импорт",
        "Import Library from File…" => "Импортировать библиотеку из файла…",
        "Import Formation from File…" => "Импортировать формирование из файла…",
        "Export" => "Экспорт",
        "Export Library…" => "Экспортировать библиотеку…",
        "Export Selected Formation…" => "Экспортировать выбранное формирование…",
        "Export as Spreadsheet…" => "Экспортировать как электронную таблицу…",
        "Export Diagram…" => "Экспортировать диаграмму…",
        "Exit" => "Выход",
        "Edit" => "Правка",
        "Find" => "Поиск",
        "Find and Replace" => "Поиск и замена",
        "Undo" => "Отменить",
        "Redo" => "Повторить",
        "Cut" => "Вырезать",
        "Copy" => "Копировать",
        "Paste" => "Вставить",
        "Delete" => "Удалить",
        "Add New Formation…" => "Добавить новое формирование…",
        "Edit Properties…" => "Редактировать свойства…",
        "Library" => "Библиотека",
        "Positions and Ranks Editor…" => "Редактор должностей и званий…",
        "Equipment and Vehicles Editor…" => "Редактор вооружения и техники…",
        "Library Properties…" => "Свойства библиотеки…",
        "Manage Tags…" => "Управление тегами…",
        "Version Control" => "Контроль версий",
        "View History…" => "Просмотреть историю…",
        "Create Snapshot (Commit)…" => "Создать снимок (Commit)…",
        "Compare Versions…" => "Сравнить версии…",
        "Revert to Version…" => "Откатить к версии…",
        "Unit" => "Формирование",
        "Add Child Formation…" => "Добавить дочернее формирование…",
        "Delete This Formation" => "Удалить это формирование",
        "Move Up" => "Переместить вверх",
        "Move Down" => "Переместить вниз",
        "Summary Table" => "Суммарная таблица",
        "Export This Formation…" => "Экспортировать это формирование…",
        "View" => "Вид",
        "View Mode" => "Режим просмотра",
        "Table" => "Таблица",
        "Diagram" => "Диаграмма",
        "Table and Diagram" => "Таблица и Диаграмма",
        "Tactical Symbols" => "Тактические знаки",
        "NATO (APP-6)" => "НАТО (APP-6)",
        "Russia (ГОСТ РВ)" => "Россия (ГОСТ РВ)",
        "Load Custom Set…" => "Загрузить набор…",
        "Color Scheme…" => "Цветовая схема…",
        "Show Equipment Images" => "Показывать изображения техники",
        "Zoom" => "Масштаб",
        "Zoom In" => "Увеличить",
        "Zoom Out" => "Уменьшить",
        "Reset Zoom" => "Сбросить (100%)",
        "Refresh" => "Обновить",
        "Tools" => "Инструменты",
        "Settings…" => "Настройки…",
        "Interface Language" => "Язык интерфейса",
        "English" => "Английский",
        "Russian" => "Русский",
        "Data Paths…" => "Пути к данным…",
        "Reset Settings" => "Сброс настроек",
        "Help" => "Справка",
        "User Guide" => "Руководство пользователя",
        "About TOEditor…" => "О программе…",
        "Check for Updates" => "Проверить обновления",
        "Delete library?" => "Удалить библиотеку?",
        "Cancel" => "Отмена",
        "Delete library \"{}\"? This will delete all versions." => "Удалить библиотеку \"{}\"? Будут удалены все версии.",
        "Formation levels" => "Уровни формирований",
        "Formation levels…" => "Уровни формирований…",
        "Name (Russian)" => "Название (рус.)",
        "Name (English)" => "Название (англ.)",
        "Corresponds to" => "Соответствует уровню",
        "Add level" => "Добавить уровень",
        "Delete level" => "Удалить уровень",
        "Export…" => "Экспорт…",
        "Import…" => "Импорт…",
        "Copy from library" => "Копировать из библиотеки",
        "Close" => "Закрыть",
        "Branches…" => "Роды войск…",
        "Branches of service" => "Роды войск",
        "Branch categories…" => "Категории родов войск…",
        "Branch categories" => "Категории родов войск",
        "Category" => "Категория",
        "Add" => "Добавить",
        _ => key,
    };
    s.to_string()
}

/// Set all UI string properties from Rust so they update when language changes.
fn apply_ui_translations(window: &MainWindow, lang: &str) {
    window.set_tr_new_library(ui_tr(lang, "New Library").into());
    window.set_tr_open_library(ui_tr(lang, "Open Library").into());
    window.set_tr_save_library(ui_tr(lang, "Save Library").into());
    window.set_tr_language(ui_tr(lang, "Language").into());
    window.set_tr_libraries(ui_tr(lang, "Libraries").into());
    window.set_tr_welcome_title(ui_tr(lang, "Welcome to TOEditor").into());
    window.set_tr_welcome_desc(
        ui_tr(lang, "Create a new library or open an existing one to get started.").into(),
    );
    // Menu
    window.set_tr_file(ui_tr(lang, "File").into());
    window.set_tr_open_library_ellipsis(ui_tr(lang, "Open Library…").into());
    window.set_tr_recent_libraries(ui_tr(lang, "Recent Libraries").into());
    window.set_tr_save_library_as(ui_tr(lang, "Save Library As…").into());
    window.set_tr_import(ui_tr(lang, "Import").into());
    window.set_tr_import_library_from_file(ui_tr(lang, "Import Library from File…").into());
    window.set_tr_import_formation_from_file(ui_tr(lang, "Import Formation from File…").into());
    window.set_tr_export(ui_tr(lang, "Export").into());
    window.set_tr_export_library_ellipsis(ui_tr(lang, "Export Library…").into());
    window.set_tr_export_selected_formation(ui_tr(lang, "Export Selected Formation…").into());
    window.set_tr_export_as_spreadsheet(ui_tr(lang, "Export as Spreadsheet…").into());
    window.set_tr_export_diagram(ui_tr(lang, "Export Diagram…").into());
    window.set_tr_exit(ui_tr(lang, "Exit").into());
    window.set_tr_edit(ui_tr(lang, "Edit").into());
    window.set_tr_find(ui_tr(lang, "Find").into());
    window.set_tr_find_and_replace(ui_tr(lang, "Find and Replace").into());
    window.set_tr_undo(ui_tr(lang, "Undo").into());
    window.set_tr_redo(ui_tr(lang, "Redo").into());
    window.set_tr_cut(ui_tr(lang, "Cut").into());
    window.set_tr_copy(ui_tr(lang, "Copy").into());
    window.set_tr_paste(ui_tr(lang, "Paste").into());
    window.set_tr_delete(ui_tr(lang, "Delete").into());
    window.set_tr_add_new_formation(ui_tr(lang, "Add New Formation…").into());
    window.set_tr_edit_properties(ui_tr(lang, "Edit Properties…").into());
    window.set_tr_library(ui_tr(lang, "Library").into());
    window.set_tr_positions_and_ranks_editor(ui_tr(lang, "Positions and Ranks Editor…").into());
    window.set_tr_equipment_and_vehicles_editor(ui_tr(lang, "Equipment and Vehicles Editor…").into());
    window.set_tr_formation_levels(ui_tr(lang, "Formation levels…").into());
    window.set_tr_branches(ui_tr(lang, "Branches…").into());
    window.set_tr_branch_categories(ui_tr(lang, "Branch categories…").into());
    window.set_tr_library_properties(ui_tr(lang, "Library Properties…").into());
    window.set_tr_manage_tags(ui_tr(lang, "Manage Tags…").into());
    window.set_tr_version_control(ui_tr(lang, "Version Control").into());
    window.set_tr_view_history(ui_tr(lang, "View History…").into());
    window.set_tr_create_snapshot(ui_tr(lang, "Create Snapshot (Commit)…").into());
    window.set_tr_compare_versions(ui_tr(lang, "Compare Versions…").into());
    window.set_tr_revert_to_version(ui_tr(lang, "Revert to Version…").into());
    window.set_tr_unit(ui_tr(lang, "Unit").into());
    window.set_tr_add_child_formation(ui_tr(lang, "Add Child Formation…").into());
    window.set_tr_delete_this_formation(ui_tr(lang, "Delete This Formation").into());
    window.set_tr_move_up(ui_tr(lang, "Move Up").into());
    window.set_tr_move_down(ui_tr(lang, "Move Down").into());
    window.set_tr_summary_table(ui_tr(lang, "Summary Table").into());
    window.set_tr_export_this_formation(ui_tr(lang, "Export This Formation…").into());
    window.set_tr_view(ui_tr(lang, "View").into());
    window.set_tr_view_mode(ui_tr(lang, "View Mode").into());
    window.set_tr_table(ui_tr(lang, "Table").into());
    window.set_tr_diagram(ui_tr(lang, "Diagram").into());
    window.set_tr_table_and_diagram(ui_tr(lang, "Table and Diagram").into());
    window.set_tr_tactical_symbols(ui_tr(lang, "Tactical Symbols").into());
    window.set_tr_nato_app6(ui_tr(lang, "NATO (APP-6)").into());
    window.set_tr_russia_gost(ui_tr(lang, "Russia (ГОСТ РВ)").into());
    window.set_tr_load_custom_set(ui_tr(lang, "Load Custom Set…").into());
    window.set_tr_color_scheme(ui_tr(lang, "Color Scheme…").into());
    window.set_tr_light(ui_tr(lang, "Light").into());
    window.set_tr_dark(ui_tr(lang, "Dark").into());
    window.set_tr_show_equipment_images(ui_tr(lang, "Show Equipment Images").into());
    window.set_tr_zoom(ui_tr(lang, "Zoom").into());
    window.set_tr_zoom_in(ui_tr(lang, "Zoom In").into());
    window.set_tr_zoom_out(ui_tr(lang, "Zoom Out").into());
    window.set_tr_reset_zoom(ui_tr(lang, "Reset Zoom").into());
    window.set_tr_refresh(ui_tr(lang, "Refresh").into());
    window.set_tr_tools(ui_tr(lang, "Tools").into());
    window.set_tr_settings(ui_tr(lang, "Settings…").into());
    window.set_tr_interface_language(ui_tr(lang, "Interface Language").into());
    window.set_tr_english(ui_tr(lang, "English").into());
    window.set_tr_russian(ui_tr(lang, "Russian").into());
    window.set_tr_data_paths(ui_tr(lang, "Data Paths…").into());
    window.set_tr_reset_settings(ui_tr(lang, "Reset Settings").into());
    window.set_tr_help(ui_tr(lang, "Help").into());
    window.set_tr_user_guide(ui_tr(lang, "User Guide").into());
    window.set_tr_about_toeditor(ui_tr(lang, "About TOEditor…").into());
    window.set_tr_check_for_updates(ui_tr(lang, "Check for Updates").into());
}

/// Application state shared between callbacks
struct AppState {
    database: Option<Database>,
    current_library: Option<Library>,
}

/// Main application window structure
pub struct AppMainWindow {
    window: MainWindow,
    translation_manager: TranslationManager,
    settings: Settings,
    state: Rc<RefCell<AppState>>,
}

impl AppMainWindow {
    /// Create new main window
    pub fn new() -> Result<Self> {
        let window = MainWindow::new()?;
        let mut translation_manager = TranslationManager::new();
        let mut settings = crate::config::Settings::load().unwrap_or_default();
        
        // Load language from settings
        let lang = Language::from_code(&settings.language);
        translation_manager.set_language(lang);
        
        // Try to set initial translation
        let lang_code = lang.code();
        if let Err(e) = slint::select_bundled_translation(lang_code) {
            eprintln!("Warning: Could not set initial translation: {}", e);
        }
        
        // Initialize database first
        let db_path = settings.database_path.clone()
            .unwrap_or_else(|| crate::config::Settings::default_database_path().unwrap_or_default());
        let database = if db_path.exists() || db_path.parent().map(|p| p.exists()).unwrap_or(false) {
            match crate::db::Database::open(&db_path) {
                Ok(db) => {
                    eprintln!("[INIT] Database opened: {:?}", db_path);
                    Some(db)
                }
                Err(e) => {
                    eprintln!("[WARNING] Failed to open database: {}", e);
                    None
                }
            }
        } else {
            // Create new database
            match crate::db::Database::open(&db_path) {
                Ok(db) => {
                    eprintln!("[INIT] Database created: {:?}", db_path);
                    Some(db)
                }
                Err(e) => {
                    eprintln!("[WARNING] Failed to create database: {}", e);
                    None
                }
            }
        };
        
        let state = Rc::new(RefCell::new(AppState {
            database,
            current_library: None,
        }));
        
        // Set initial theme from settings
        let theme = if settings.color_scheme == "dark" { "dark" } else { "light" };
        window.set_theme(theme.into());
        eprintln!("[INIT] Initial theme set to: {}", theme);
        
        // Set up UI callbacks FIRST, before any other initialization
        eprintln!("[INIT] Setting up callbacks...");
        setup_callbacks_with_state(&window, &mut translation_manager, &mut settings, state.clone())?;
        eprintln!("[INIT] Callbacks set up successfully");
        
        // Initialize toolbar
        init_toolbar(&window)?;
        
        // Set window title
        let version = env!("CARGO_PKG_VERSION");
        window.set_window_title(format!("TOEditor v{}", version).into());
        
        // Set initial language property and UI strings from Rust
        window.set_current_language(lang_code.into());
        apply_ui_translations(&window, lang_code);
        eprintln!("[INIT] Initial language set to: {}", lang_code);
        
        // Load libraries into UI
        refresh_libraries_list(&window, state.clone());
        
        // Initialize database
        let db_path = settings.database_path.clone()
            .unwrap_or_else(|| crate::config::Settings::default_database_path().unwrap_or_default());
        let database = if db_path.exists() || db_path.parent().map(|p| p.exists()).unwrap_or(false) {
            match crate::db::Database::open(&db_path) {
                Ok(db) => {
                    eprintln!("[INIT] Database opened: {:?}", db_path);
                    Some(db)
                }
                Err(e) => {
                    eprintln!("[WARNING] Failed to open database: {}", e);
                    None
                }
            }
        } else {
            // Create new database
            match crate::db::Database::open(&db_path) {
                Ok(db) => {
                    eprintln!("[INIT] Database created: {:?}", db_path);
                    Some(db)
                }
                Err(e) => {
                    eprintln!("[WARNING] Failed to create database: {}", e);
                    None
                }
            }
        };
        
        let state = Rc::new(RefCell::new(AppState {
            database,
            current_library: None,
        }));
        
        // Store state reference for callbacks
        let state_for_callbacks = state.clone();
        
        // Re-setup callbacks with state access
        setup_callbacks_with_state(&window, &mut translation_manager, &mut settings, state_for_callbacks)?;
        
        Ok(Self { 
            window, 
            translation_manager, 
            settings,
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
    _translation_manager: &mut TranslationManager,
    settings: &mut crate::config::Settings,
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
            // Write to both stderr and a log file for debugging
            let debug_msg = format!("[DEBUG] ===== Language switch callback INVOKED with: {} =====\n", lang_code);
            eprint!("{}", debug_msg);
            
            // Also write to a log file
            if let Err(e) = std::fs::write("language_switch.log", &debug_msg) {
                eprintln!("[WARNING] Could not write to log file: {}", e);
            }
            
            if let Some(window) = weak.upgrade() {
                let lang = Language::from_code(&lang_code.to_string());
                eprintln!("[DEBUG] Parsed language: {} ({})", lang.name(), lang.code());
                
                // Update settings
                let mut settings = match crate::config::Settings::load() {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("[ERROR] Failed to load settings: {}", e);
                        crate::config::Settings::default()
                    }
                };
                settings.language = lang.code().to_string();
                
                match settings.save() {
                    Ok(_) => eprintln!("[DEBUG] Settings saved successfully"),
                    Err(e) => eprintln!("[ERROR] Failed to save settings: {}", e),
                }
                
                // Update the language property
                window.set_current_language(lang_code.clone());
                eprintln!("[DEBUG] Language property set to: {}", lang_code);
                
                // Try to switch translation if bundled translations are available
                match slint::select_bundled_translation(&lang_code.to_string()) {
                    Ok(_) => eprintln!("[DEBUG] Translation API call succeeded"),
                    Err(e) => eprintln!("[WARNING] Translation API call failed (expected if not bundled): {}", e),
                }
                
                // Update title and all UI strings from Rust
                let version = env!("CARGO_PKG_VERSION");
                let lang_name = lang.name();
                let new_title = format!("TOEditor v{} [{}]", version, lang_name);
                window.set_window_title(new_title.clone().into());
                apply_ui_translations(&window, &lang_code.to_string());
                window.window().request_redraw();
                eprintln!("[DEBUG] Window title updated to: {}", new_title);
                
                eprintln!("[DEBUG] ===== Language switch COMPLETE =====");
            } else {
                eprintln!("[ERROR] ===== FAILED: Could not upgrade weak window reference =====");
            }
        }
    });
    
    // Also register separate callbacks for each language as a workaround
    // These will be called directly from MenuItem, and then invoke the main callback
    window.on_switch_to_english({
        let weak = weak_window.clone();
        move || {
            let debug_msg = "[DEBUG] ===== switch-to-english callback INVOKED =====\n";
            eprint!("{}", debug_msg);
            let _ = std::fs::write("language_switch.log", debug_msg);
            
            if let Some(window) = weak.upgrade() {
                // Directly call the language switch logic
                let lang_code: SharedString = "en".into();
                window.invoke_switch_language(lang_code.clone());
                
                // Also manually trigger the logic as fallback
                let mut settings = crate::config::Settings::load().unwrap_or_default();
                settings.language = "en".to_string();
                let _ = settings.save();
                window.set_current_language(lang_code);
                
                let version = env!("CARGO_PKG_VERSION");
                window.set_window_title(format!("TOEditor v{} [English]", version).into());
                apply_ui_translations(&window, "en");
                window.window().request_redraw();
                eprintln!("[DEBUG] English language set via direct callback");
            }
        }
    });
    
    window.on_switch_to_russian({
        let weak = weak_window.clone();
        move || {
            let debug_msg = "[DEBUG] ===== switch-to-russian callback INVOKED =====\n";
            eprint!("{}", debug_msg);
            let _ = std::fs::write("language_switch.log", debug_msg);
            
            if let Some(window) = weak.upgrade() {
                // Directly call the language switch logic
                let lang_code: SharedString = "ru".into();
                window.invoke_switch_language(lang_code.clone());
                
                // Also manually trigger the logic as fallback
                let mut settings = crate::config::Settings::load().unwrap_or_default();
                settings.language = "ru".to_string();
                let _ = settings.save();
                window.set_current_language(lang_code);
                
                let version = env!("CARGO_PKG_VERSION");
                window.set_window_title(format!("TOEditor v{} [Русский]", version).into());
                apply_ui_translations(&window, "ru");
                window.window().request_redraw();
                eprintln!("[DEBUG] Russian language set via direct callback");
            }
        }
    });
    
    eprintln!("[DEBUG] All language switch callbacks registered successfully");
    
    // File menu actions
    window.on_file_exit({
        let weak = weak_window.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                eprintln!("[DEBUG] File > Exit called");
                window.hide().unwrap_or_default();
                std::process::exit(0);
            }
        }
    });
    
    // Library management handlers
    let weak_window = window.as_weak();
    window.on_file_new_library(move || {
        eprintln!("[DEBUG] File > New Library");
        if let Some(window) = weak_window.upgrade() {
            show_library_dialog(&window, "new", -1);
        }
    });
    
    // Library dialog handlers
    let state_clone = state.clone();
    let weak_window = window.as_weak();
    window.on_library_dialog_accepted(move |name: SharedString, country: SharedString, era: SharedString, author: SharedString, tags: SharedString, library_id: i32| {
        eprintln!("[DEBUG] Library dialog accepted: name={}, country={}, era={}, author={}, tags={}, id={}", 
                  name, country, era, author, tags, library_id);
        
        // Parse tags first
        let tags_vec: Vec<String> = tags.to_string()
            .split(',')
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
                            eprintln!("[INFO] Library created: {} (ID: {:?})", lib.name, lib.id);
                            drop(state);
                            let lib_id = lib.id.map(|x| x as i32).unwrap_or(-1);
                            state_clone.borrow_mut().current_library = Some(lib.clone());
                            if let Some(window) = weak_window.upgrade() {
                                window.set_current_library_name(lib.name.clone().into());
                                window.set_current_library_id(lib_id);
                                refresh_libraries_list(&window, state_clone.clone());
                            }
                        }
                        Err(e) => eprintln!("[ERROR] Failed to create library: {}", e),
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
                                eprintln!("[INFO] Library updated successfully");
                                drop(state);
                                let lib_id = lib.id.map(|x| x as i32).unwrap_or(-1);
                                state_clone.borrow_mut().current_library = Some(lib);
                                if let Some(window) = weak_window.upgrade() {
                                    window.set_current_library_name(name.clone());
                                    window.set_current_library_id(lib_id);
                                    refresh_libraries_list(&window, state_clone.clone());
                                }
                            }
                            Err(e) => eprintln!("[ERROR] Failed to update library: {}", e),
                        }
                    }
                }
            } else {
                eprintln!("[ERROR] Database not initialized");
            }
        }
    });
    
    window.on_library_dialog_cancelled(|| {
        eprintln!("[DEBUG] Library dialog cancelled");
    });
    
    let state_clone = state.clone();
    let weak_window = window.as_weak();
    window.on_file_open_library(move || {
        eprintln!("[DEBUG] File > Open Library");
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
        
        // Position menu near the clicked library item
        // Calculate position based on item index: each item is 28px high, sidebar header is ~50px
        use slint::ComponentHandle;
        let menu_window = menu.window();
        if let Some(main_window) = weak_window.upgrade() {
            let main_window_handle = main_window.window();
            let main_pos = main_window_handle.position();
            // Sidebar width when expanded is 220px, libraries list starts at y ~50px (menu bar + header)
            // Each library item is 28px high
            let item_height = 28;
            let sidebar_header_height = 50;
            let menu_x = (main_pos.x + 220) as f32; // Right edge of sidebar
            let menu_y = (main_pos.y + sidebar_header_height + (item_index * item_height)) as f32;
            menu_window.set_position(slint::WindowPosition::Logical(slint::LogicalPosition::new(menu_x, menu_y)));
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
            eprintln!("[DEBUG] Library > View history");
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
        eprintln!("[DEBUG] Library selected: {}", library_id);
        let has_db = {
            let state = state_clone.borrow();
            state.database.is_some()
        };
        
        if !has_db {
            eprintln!("[ERROR] Database not initialized");
            return;
        }
        
        {
            let state = state_clone.borrow();
            if let Some(ref db) = state.database {
                let service = LibraryService::new(db.conn());
                match service.get_library(library_id as i64) {
                    Ok(Some(lib)) => {
                        eprintln!("[INFO] Loaded library: {}", lib.name);
                        drop(state);
                        state_clone.borrow_mut().current_library = Some(lib.clone());
                        if let Some(window) = weak_window.upgrade() {
                            window.set_current_library_name(lib.name.clone().into());
                            window.set_current_library_id(library_id);
                            refresh_formations_list(&window);
                        }
                    }
                    Err(e) => eprintln!("[ERROR] Failed to load library: {}", e),
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
        eprintln!("[DEBUG] File > Recent Libraries");
        // TODO: Show list of recently opened libraries
    });
    
    let state_clone = state.clone();
    window.on_file_save_library(move || {
        eprintln!("[DEBUG] File > Save Library");
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
                        eprintln!("[INFO] Library saved successfully");
                    }
                    Err(e) => {
                        drop(state);
                        eprintln!("[ERROR] Failed to save library: {}", e);
                    }
                }
            } else {
                eprintln!("[ERROR] Database not initialized");
            }
        } else {
            eprintln!("[WARNING] No library to save. Create or open a library first.");
        }
    });
    
    window.on_file_save_library_as(|| {
        eprintln!("[DEBUG] File > Save Library As");
        // TODO: Show file dialog to save as JSON
    });
    
    let state_clone = state.clone();
    window.on_file_import_library(move || {
        eprintln!("[DEBUG] File > Import Library");
        // TODO: Show file dialog to import JSON
        // For now, demonstrate import from a test file
        let state = state_clone.borrow();
        if state.database.is_some() {
            // Example: import from temp file
            eprintln!("[INFO] Import functionality - TODO: implement file dialog");
        }
    });
    
    window.on_file_import_formation(|| { eprintln!("[DEBUG] File > Import Formation"); });
    
    let state_clone = state.clone();
    window.on_file_export_library(move || {
        eprintln!("[DEBUG] File > Export Library");
        let state = state_clone.borrow();
        if let Some(ref lib) = state.current_library {
            // Export to temp directory for now
            let path = std::env::temp_dir().join(format!("{}.json", lib.name));
            match export::export_json(lib, &path) {
                Ok(_) => eprintln!("[INFO] Library exported to: {:?}", path),
                Err(e) => eprintln!("[ERROR] Failed to export library: {}", e),
            }
        } else {
            eprintln!("[WARNING] No library to export. Create or open a library first.");
        }
    });
    
    window.on_file_export_formation(|| { eprintln!("[DEBUG] File > Export Formation"); });
    window.on_file_export_spreadsheet(|| { eprintln!("[DEBUG] File > Export Spreadsheet"); });
    window.on_file_export_diagram(|| { eprintln!("[DEBUG] File > Export Diagram"); });
    
    // Edit menu actions
    window.on_edit_find(|| { eprintln!("[DEBUG] Edit > Find"); });
    window.on_edit_find_replace(|| { eprintln!("[DEBUG] Edit > Find and Replace"); });
    window.on_edit_undo(|| { eprintln!("[DEBUG] Edit > Undo"); });
    window.on_edit_redo(|| { eprintln!("[DEBUG] Edit > Redo"); });
    window.on_edit_cut(|| { eprintln!("[DEBUG] Edit > Cut"); });
    window.on_edit_copy(|| { eprintln!("[DEBUG] Edit > Copy"); });
    window.on_edit_paste(|| { eprintln!("[DEBUG] Edit > Paste"); });
    window.on_edit_delete(|| { eprintln!("[DEBUG] Edit > Delete"); });
    window.on_edit_add_formation(|| { eprintln!("[DEBUG] Edit > Add Formation"); });
    window.on_edit_edit_properties(|| { eprintln!("[DEBUG] Edit > Edit Properties"); });
    
    // Library menu actions
    window.on_library_positions_editor(|| { eprintln!("[DEBUG] Library > Positions Editor"); });
    window.on_library_equipment_editor(|| { eprintln!("[DEBUG] Library > Equipment Editor"); });
    
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
                        eprintln!("[WARNING] Library has no id");
                        return;
                    }
                },
                None => {
                    eprintln!("[WARNING] No library selected");
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
                        eprintln!("[WARNING] Library has no id");
                        return;
                    }
                },
                None => {
                    eprintln!("[WARNING] No library selected");
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
                        eprintln!("[WARNING] Library has no id");
                        return;
                    }
                },
                None => {
                    eprintln!("[WARNING] No library selected");
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
        eprintln!("[DEBUG] Library > Properties");
        let state = state_clone.borrow();
        if let Some(ref lib) = state.current_library {
                if let Some(lib_id) = lib.id {
                drop(state);
                if let Some(window) = weak_window.upgrade() {
                    show_library_dialog_for_edit(&window, lib_id as i32, state_clone.clone());
                }
            }
        } else {
            eprintln!("[WARNING] No library selected");
        }
    });
    
    window.on_library_manage_tags(|| { eprintln!("[DEBUG] Library > Manage Tags"); });
    window.on_library_export_library(|| { eprintln!("[DEBUG] Library > Export Library"); });
    window.on_library_view_history(|| { eprintln!("[DEBUG] Library > View History"); });
    window.on_library_create_snapshot(|| { eprintln!("[DEBUG] Library > Create Snapshot"); });
    window.on_library_compare_versions(|| { eprintln!("[DEBUG] Library > Compare Versions"); });
    window.on_library_revert_to_version(|| { eprintln!("[DEBUG] Library > Revert to Version"); });
    
    // Library delete: show confirmation dialog, then delete on confirm
    let state_clone = state.clone();
    let weak_window = window.as_weak();
    window.on_library_delete(move || {
        eprintln!("[DEBUG] Library > Delete");
        let (lib_id, lib_name) = {
            let state = state_clone.borrow();
            match &state.current_library {
                Some(lib) => match lib.id {
                    Some(id) => (id, lib.name.clone()),
                    None => {
                        eprintln!("[WARNING] Library has no id");
                        return;
                    }
                },
                None => {
                    eprintln!("[WARNING] No library selected");
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
                eprintln!("[ERROR] Failed to create confirm dialog: {}", e);
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
                            eprintln!("[ERROR] Failed to delete library: {}", e);
                            false
                        }
                    }
                } else {
                    false
                }
            };
            if delete_ok {
                eprintln!("[INFO] Library deleted successfully");
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
    window.on_unit_add_child(|| { eprintln!("[DEBUG] Unit > Add Child"); });
    window.on_unit_delete(|| { eprintln!("[DEBUG] Unit > Delete"); });
    window.on_unit_move_up(|| { eprintln!("[DEBUG] Unit > Move Up"); });
    window.on_unit_move_down(|| { eprintln!("[DEBUG] Unit > Move Down"); });
    window.on_unit_summary_table(|| { eprintln!("[DEBUG] Unit > Summary Table"); });
    window.on_unit_export(|| { eprintln!("[DEBUG] Unit > Export"); });
    window.on_unit_view_history(|| { eprintln!("[DEBUG] Unit > View History"); });
    window.on_unit_create_snapshot(|| { eprintln!("[DEBUG] Unit > Create Snapshot"); });
    window.on_unit_compare_versions(|| { eprintln!("[DEBUG] Unit > Compare Versions"); });
    window.on_unit_revert_to_version(|| { eprintln!("[DEBUG] Unit > Revert to Version"); });
    
    // View menu actions
    window.on_view_table(|| { eprintln!("[DEBUG] View > Table"); });
    window.on_view_diagram(|| { eprintln!("[DEBUG] View > Diagram"); });
    window.on_view_table_and_diagram(|| { eprintln!("[DEBUG] View > Table and Diagram"); });
    window.on_view_symbols_nato(|| { eprintln!("[DEBUG] View > Symbols NATO"); });
    window.on_view_symbols_russia(|| { eprintln!("[DEBUG] View > Symbols Russia"); });
    window.on_view_load_symbols(|| { eprintln!("[DEBUG] View > Load Symbols"); });
    // Theme switching callback
    let weak_window = window.as_weak();
    window.on_switch_theme(move |theme: slint::SharedString| {
        let theme_str = theme.to_string();
        eprintln!("[DEBUG] Switching theme to: {}", theme_str);
        
        if let Some(w) = weak_window.upgrade() {
            w.set_theme(theme.clone());
            
            // Save theme to settings
            let mut settings = match crate::config::Settings::load() {
                Ok(s) => s,
                Err(_) => crate::config::Settings::default(),
            };
            settings.color_scheme = theme_str.clone();
            if let Err(e) = settings.save() {
                eprintln!("[ERROR] Failed to save theme setting: {}", e);
            } else {
                eprintln!("[INFO] Theme saved: {}", theme_str);
            }
        }
    });
    window.on_view_show_images(|| { eprintln!("[DEBUG] View > Show Images"); });
    window.on_view_zoom_in(|| { eprintln!("[DEBUG] View > Zoom In"); });
    window.on_view_zoom_out(|| { eprintln!("[DEBUG] View > Zoom Out"); });
    window.on_view_zoom_reset(|| { eprintln!("[DEBUG] View > Zoom Reset"); });
    window.on_view_refresh(|| { eprintln!("[DEBUG] View > Refresh"); });
    
    // Tools menu actions
    window.on_tools_settings(|| { eprintln!("[DEBUG] Tools > Settings"); });
    window.on_tools_language(|| { eprintln!("[DEBUG] Tools > Language"); });
    window.on_tools_data_paths(|| { eprintln!("[DEBUG] Tools > Data Paths"); });
    window.on_tools_reset_settings(|| { eprintln!("[DEBUG] Tools > Reset Settings"); });
    
    // Help menu actions
    window.on_help_user_guide(|| { eprintln!("[DEBUG] Help > User Guide"); });
    window.on_help_about(|| { eprintln!("[DEBUG] Help > About"); });
    window.on_help_check_updates(|| { eprintln!("[DEBUG] Help > Check Updates"); });
    
    Ok(())
}

/// Show library dialog for creating new library
fn show_library_dialog(window: &MainWindow, _mode: &str, library_id: i32) {
    use slint::ComponentHandle;
    
    let dialog = match LibraryDialog::new() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[ERROR] Failed to create library dialog: {}", e);
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

/// Show library dialog for editing existing library
fn show_library_dialog_for_edit(window: &MainWindow, library_id: i32, state: Rc<RefCell<AppState>>) {
    use slint::ComponentHandle;
    
    let dialog = match LibraryDialog::new() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[ERROR] Failed to create library dialog: {}", e);
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

/// Open the Branches editor window for the given library.
fn show_branches_editor(state: Rc<RefCell<AppState>>, lib_id: i64, lib_name: &str, lang: &str) {
    use slint::ComponentHandle;
    let (branches, other_library_items, source_library_ids, category_items) = {
        let st = state.borrow();
        let db = match st.database.as_ref() {
            Some(d) => d,
            None => {
                eprintln!("[ERROR] Database not initialized");
                return;
            }
        };
        let branch_repo = BranchRepo::new(db.conn());
        let mut branches = match branch_repo.list_by_library(lib_id) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("[ERROR] Failed to load branches: {}", e);
                return;
            }
        };
        if branches.is_empty() {
            let cat_repo = crate::db::repositories::BranchCategoryRepo::new(db.conn());
            let mut category_ids = Vec::new();
            for mut cat in crate::models::default_branch_categories(lib_id) {
                if cat_repo.create(&mut cat).is_err() {
                    break;
                }
                if let Some(id) = cat.id {
                    category_ids.push(id);
                }
            }
            for (mut b, cat_idx) in crate::models::default_branches(lib_id) {
                b.category_id = category_ids.get(cat_idx).copied();
                if branch_repo.create(&mut b).is_err() {
                    break;
                }
            }
            branches = branch_repo.list_by_library(lib_id).unwrap_or_default();
        }
        let lib_repo = crate::db::repositories::LibraryRepo::new(db.conn());
        let all_libs = match lib_repo.list_all() {
            Ok(l) => l,
            Err(_) => vec![],
        };
        let mut other_items = Vec::new();
        let mut source_ids = Vec::new();
        for l in all_libs {
            if l.id != Some(lib_id) {
                if let Some(id) = l.id {
                    other_items.push(OtherLibraryItem { id: id as i32, name: l.name.into() });
                    source_ids.push(id);
                }
            }
        }
        let categories = BranchCategoryRepo::new(db.conn()).list_by_library(lib_id).unwrap_or_default();
        let category_items: Vec<CategoryItem> = categories
            .iter()
            .map(|c| CategoryItem {
                id: c.id.unwrap_or(-1) as i32,
                name: if lang == "ru" { c.name_ru.as_str() } else { c.name_en.as_str() }.into(),
            })
            .collect();
        (branches, other_items, source_ids, category_items)
    };
    let category_items_clone = category_items.clone();
    let rows: Vec<BranchRow> = branches
        .into_iter()
        .map(|b| BranchRow {
            id: b.id.unwrap_or(-1) as i32,
            category_id: b.category_id.unwrap_or(-1) as i32,
            name_ru: b.name_ru.into(),
            name_en: b.name_en.into(),
        })
        .collect();
    let editor = match BranchesEditor::new() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("[ERROR] Failed to create Branches editor: {}", e);
            return;
        }
    };
    editor.set_library_id(lib_id as i32);
    editor.set_library_name(lib_name.into());
    let model = Rc::new(VecModel::from(rows));
    editor.set_branches(ModelRc::new(model.clone()));
    editor.set_current_index(-1);
    editor.set_current_name_ru(Default::default());
    editor.set_current_name_en(Default::default());
    editor.set_tr_branches_title(ui_tr(lang, "Branches of service").into());
    editor.set_tr_name_russian(ui_tr(lang, "Name (Russian)").into());
    editor.set_tr_name_english(ui_tr(lang, "Name (English)").into());
    editor.set_tr_category(ui_tr(lang, "Category").into());
    editor.set_categories(ModelRc::new(VecModel::from(category_items.clone())));
    editor.set_current_category_index(-1);
    editor.set_tr_add(ui_tr(lang, "Add").into());
    editor.set_tr_delete(ui_tr(lang, "Delete").into());
    editor.set_tr_export(ui_tr(lang, "Export…").into());
    editor.set_tr_import(ui_tr(lang, "Import…").into());
    editor.set_tr_copy_from_library(ui_tr(lang, "Copy from library").into());
    editor.set_tr_close(ui_tr(lang, "Close").into());
    editor.set_other_libraries(ModelRc::new(VecModel::from(other_library_items)));
    editor.set_copy_source_index(-1);
    let state_close = state.clone();
    let weak_editor = editor.as_weak();
    let weak_add = weak_editor.clone();
    let model_add = model.clone();
    editor.on_add_branch(move || {
        let Some(ed) = weak_add.upgrade() else { return };
        let row = BranchRow { id: -1, category_id: -1, name_ru: Default::default(), name_en: Default::default() };
        model_add.insert(model_add.row_count(), row.clone());
        ed.set_current_index(model_add.row_count() as i32 - 1);
        ed.set_current_name_ru(row.name_ru.clone());
        ed.set_current_name_en(row.name_en.clone());
        ed.set_current_category_index(-1);
    });
    let weak_del = weak_editor.clone();
    let model_del = model.clone();
    let category_items_del = category_items_clone.clone();
    editor.on_delete_branch(move || {
        let Some(ed) = weak_del.upgrade() else { return };
        let idx = ed.get_current_index();
        if idx >= 0 && (idx as usize) < model_del.row_count() {
            model_del.remove(idx as usize);
            let new_count = model_del.row_count();
            if new_count == 0 {
                ed.set_current_index(-1);
                ed.set_current_name_ru(Default::default());
                ed.set_current_name_en(Default::default());
                ed.set_current_category_index(-1);
            } else {
                let new_idx = (idx as usize).min(new_count - 1);
                ed.set_current_index(new_idx as i32);
                if let Some(r) = model_del.row_data(new_idx) {
                    ed.set_current_name_ru(r.name_ru.clone());
                    ed.set_current_name_en(r.name_en.clone());
                    let cat_idx = if r.category_id > 0 {
                        category_items_del.iter().position(|c| c.id == r.category_id).map(|i| i as i32).unwrap_or(-1)
                    } else { -1 };
                    ed.set_current_category_index(cat_idx);
                }
            }
        }
    });
    let weak_close = weak_editor.clone();
    let model_close = model.clone();
    editor.on_close_editor(move || {
        let Some(ed) = weak_close.upgrade() else { return };
        let idx = ed.get_current_index();
        if idx >= 0 && (idx as usize) < model_close.row_count() {
            let ru = ed.get_current_name_ru();
            let en = ed.get_current_name_en();
            if let Some(r) = model_close.row_data(idx as usize) {
                let _ = model_close.set_row_data(idx as usize, BranchRow {
                    id: r.id,
                    category_id: r.category_id,
                    name_ru: ru,
                    name_en: en,
                });
            }
        }
        let st = state_close.borrow();
        if let Some(ref db) = st.database {
            let repo = BranchRepo::new(db.conn());
            let _ = repo.delete_by_library(lib_id);
            for i in 0..model_close.row_count() {
                if let Some(r) = model_close.row_data(i) {
                    let cat_id = if r.category_id > 0 { Some(r.category_id as i64) } else { None };
                    let mut b = Branch::with_category(lib_id, cat_id, r.name_ru.to_string(), r.name_en.to_string());
                    let _ = repo.create(&mut b);
                }
            }
        }
        let _ = ed.hide();
    });
    let weak_sel = weak_editor.clone();
    let model_sel = model.clone();
    let category_items_sel = category_items_clone.clone();
    editor.on_selection_changed(move |index| {
        let Some(ed) = weak_sel.upgrade() else { return };
        if index >= 0 && (index as usize) < model_sel.row_count() {
            if let Some(r) = model_sel.row_data(index as usize) {
                ed.set_current_name_ru(r.name_ru.clone());
                ed.set_current_name_en(r.name_en.clone());
                let cat_idx = if r.category_id > 0 {
                    category_items_sel.iter().position(|c| c.id == r.category_id).map(|i| i as i32).unwrap_or(-1)
                } else {
                    -1
                };
                ed.set_current_category_index(cat_idx);
            }
        }
    });
    let category_items_cat = category_items_clone.clone();
    let weak_cat = weak_editor.clone();
    let model_cat = model.clone();
    editor.on_category_changed(move |index| {
        let Some(ed) = weak_cat.upgrade() else { return };
        let idx = ed.get_current_index();
        if idx >= 0 && (idx as usize) < model_cat.row_count() && index >= 0 && (index as usize) < category_items_cat.len() {
            if let Some(r) = model_cat.row_data(idx as usize) {
                let new_cat_id = category_items_cat[index as usize].id;
                let _ = model_cat.set_row_data(idx as usize, BranchRow {
                    id: r.id,
                    category_id: new_cat_id,
                    name_ru: r.name_ru.clone(),
                    name_en: r.name_en.clone(),
                });
                ed.set_current_category_index(index);
            }
        }
    });
    let model_exp = model.clone();
    editor.on_export_branches(move || {
        let branches: Vec<Branch> = (0..model_exp.row_count())
            .filter_map(|i| model_exp.row_data(i))
            .map(|r| Branch::with_category(lib_id, None, r.name_ru.to_string(), r.name_en.to_string()))
            .collect();
        if let Some(path) = rfd::FileDialog::new().add_filter("JSON", &["json"]).save_file() {
            if let Err(e) = export_branches_to_path(path.as_path(), &branches) {
                eprintln!("[ERROR] Export branches: {}", e);
            }
        }
    });
    let weak_imp = weak_editor.clone();
    let model_imp = model.clone();
    editor.on_import_branches(move || {
        if let Some(path) = rfd::FileDialog::new().add_filter("JSON", &["json"]).pick_file() {
            match import_branches_from_path(path.as_path()) {
                Ok(imported) => {
                    while model_imp.row_count() > 0 {
                        model_imp.remove(0);
                    }
                    for e in imported {
                        model_imp.insert(model_imp.row_count(), BranchRow {
                            id: -1,
                            category_id: -1,
                            name_ru: e.name_ru.into(),
                            name_en: e.name_en.into(),
                        });
                    }
                    if let Some(ed) = weak_imp.upgrade() {
                        ed.set_current_index(if model_imp.row_count() > 0 { 0 } else { -1 });
                        if model_imp.row_count() > 0 {
                            if let Some(r) = model_imp.row_data(0) {
                                ed.set_current_name_ru(r.name_ru.clone());
                                ed.set_current_name_en(r.name_en.clone());
                            }
                        }
                    }
                }
                Err(e) => eprintln!("[ERROR] Import branches: {}", e),
            }
        }
    });
    let state_copy = state.clone();
    let weak_copy = weak_editor.clone();
    let model_copy = model.clone();
    let source_ids = source_library_ids.clone();
    editor.on_copy_from_library(move || {
        let Some(ed) = weak_copy.upgrade() else { return };
        let idx = ed.get_copy_source_index();
        if idx < 0 || (idx as usize) >= source_ids.len() {
            return;
        }
        let source_id = source_ids[idx as usize];
        let st = state_copy.borrow();
        if let Some(ref db) = st.database {
            let branch_repo = BranchRepo::new(db.conn());
            if let Err(e) = copy_branches_between_libraries(&branch_repo, source_id, lib_id) {
                eprintln!("[ERROR] Copy branches: {}", e);
                return;
            }
            drop(st);
            let st2 = state_copy.borrow();
            if let Some(ref db2) = st2.database {
                let branch_repo2 = BranchRepo::new(db2.conn());
                if let Ok(new_branches) = branch_repo2.list_by_library(lib_id) {
                    while model_copy.row_count() > 0 {
                        model_copy.remove(0);
                    }
                    for b in new_branches {
                        model_copy.insert(model_copy.row_count(), BranchRow {
                            id: b.id.unwrap_or(-1) as i32,
                            category_id: b.category_id.unwrap_or(-1) as i32,
                            name_ru: b.name_ru.into(),
                            name_en: b.name_en.into(),
                        });
                    }
                    ed.set_current_index(if model_copy.row_count() > 0 { 0 } else { -1 });
                    if model_copy.row_count() > 0 {
                        if let Some(r) = model_copy.row_data(0) {
                            ed.set_current_name_ru(r.name_ru.clone());
                            ed.set_current_name_en(r.name_en.clone());
                        }
                    }
                }
            }
        }
    });
    editor.show().unwrap_or_default();
}

/// Open the Branch categories editor window for the given library.
fn show_branch_categories_editor(state: Rc<RefCell<AppState>>, lib_id: i64, lib_name: &str, lang: &str) {
    use slint::ComponentHandle;
    let (categories, other_library_items, source_library_ids) = {
        let st = state.borrow();
        let db = match st.database.as_ref() {
            Some(d) => d,
            None => {
                eprintln!("[ERROR] Database not initialized");
                return;
            }
        };
        let cat_repo = BranchCategoryRepo::new(db.conn());
        let mut categories = cat_repo.list_by_library(lib_id).unwrap_or_default();
        if categories.is_empty() {
            for mut c in crate::models::default_branch_categories(lib_id) {
                let _ = cat_repo.create(&mut c);
            }
            categories = cat_repo.list_by_library(lib_id).unwrap_or_default();
        }
        let lib_repo = crate::db::repositories::LibraryRepo::new(db.conn());
        let all_libs = lib_repo.list_all().unwrap_or_default();
        let mut other_items = Vec::new();
        let mut source_ids = Vec::new();
        for l in all_libs {
            if l.id != Some(lib_id) {
                if let Some(id) = l.id {
                    other_items.push(OtherLibraryItem { id: id as i32, name: l.name.into() });
                    source_ids.push(id);
                }
            }
        }
        (categories, other_items, source_ids)
    };
    let rows: Vec<CategoryRow> = categories
        .into_iter()
        .map(|c| CategoryRow {
            id: c.id.unwrap_or(-1) as i32,
            name_ru: c.name_ru.into(),
            name_en: c.name_en.into(),
        })
        .collect();
    let editor = match BranchCategoriesEditor::new() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("[ERROR] Failed to create Branch categories editor: {}", e);
            return;
        }
    };
    editor.set_library_id(lib_id as i32);
    editor.set_library_name(lib_name.into());
    let model = Rc::new(VecModel::from(rows));
    editor.set_categories(ModelRc::new(model.clone()));
    editor.set_current_index(-1);
    editor.set_current_name_ru(Default::default());
    editor.set_current_name_en(Default::default());
    editor.set_tr_categories_title(ui_tr(lang, "Branch categories").into());
    editor.set_tr_name_russian(ui_tr(lang, "Name (Russian)").into());
    editor.set_tr_name_english(ui_tr(lang, "Name (English)").into());
    editor.set_tr_add(ui_tr(lang, "Add").into());
    editor.set_tr_delete(ui_tr(lang, "Delete").into());
    editor.set_tr_export(ui_tr(lang, "Export…").into());
    editor.set_tr_import(ui_tr(lang, "Import…").into());
    editor.set_tr_copy_from_library(ui_tr(lang, "Copy from library").into());
    editor.set_tr_close(ui_tr(lang, "Close").into());
    editor.set_other_libraries(ModelRc::new(VecModel::from(other_library_items)));
    editor.set_copy_source_index(-1);
    let state_close = state.clone();
    let weak_editor = editor.as_weak();
    let weak_add = weak_editor.clone();
    let model_add = model.clone();
    editor.on_add_category(move || {
        let Some(ed) = weak_add.upgrade() else { return };
        let row = CategoryRow { id: -1, name_ru: Default::default(), name_en: Default::default() };
        model_add.insert(model_add.row_count(), row.clone());
        ed.set_current_index(model_add.row_count() as i32 - 1);
        ed.set_current_name_ru(row.name_ru.clone());
        ed.set_current_name_en(row.name_en.clone());
    });
    let weak_del = weak_editor.clone();
    let model_del = model.clone();
    editor.on_delete_category(move || {
        let Some(ed) = weak_del.upgrade() else { return };
        let idx = ed.get_current_index();
        if idx >= 0 && (idx as usize) < model_del.row_count() {
            model_del.remove(idx as usize);
            let new_count = model_del.row_count();
            if new_count == 0 {
                ed.set_current_index(-1);
                ed.set_current_name_ru(Default::default());
                ed.set_current_name_en(Default::default());
            } else {
                let new_idx = (idx as usize).min(new_count - 1);
                ed.set_current_index(new_idx as i32);
                if let Some(r) = model_del.row_data(new_idx) {
                    ed.set_current_name_ru(r.name_ru.clone());
                    ed.set_current_name_en(r.name_en.clone());
                }
            }
        }
    });
    let weak_close = weak_editor.clone();
    let model_close = model.clone();
    editor.on_close_editor(move || {
        let Some(ed) = weak_close.upgrade() else { return };
        let idx = ed.get_current_index();
        if idx >= 0 && (idx as usize) < model_close.row_count() {
            let ru = ed.get_current_name_ru();
            let en = ed.get_current_name_en();
            if let Some(r) = model_close.row_data(idx as usize) {
                let _ = model_close.set_row_data(idx as usize, CategoryRow {
                    id: r.id,
                    name_ru: ru,
                    name_en: en,
                });
            }
        }
        let st = state_close.borrow();
        if let Some(ref db) = st.database {
            let repo = BranchCategoryRepo::new(db.conn());
            let _ = repo.delete_by_library(lib_id);
            for i in 0..model_close.row_count() {
                if let Some(r) = model_close.row_data(i) {
                    let mut c = crate::models::BranchCategory::new(lib_id, r.name_ru.to_string(), r.name_en.to_string());
                    let _ = repo.create(&mut c);
                }
            }
        }
        let _ = ed.hide();
    });
    let weak_sel = weak_editor.clone();
    let model_sel = model.clone();
    editor.on_selection_changed(move |index| {
        let Some(ed) = weak_sel.upgrade() else { return };
        if index >= 0 && (index as usize) < model_sel.row_count() {
            if let Some(r) = model_sel.row_data(index as usize) {
                ed.set_current_name_ru(r.name_ru.clone());
                ed.set_current_name_en(r.name_en.clone());
            }
        }
    });
    let model_exp = model.clone();
    editor.on_export_categories(move || {
        let categories: Vec<crate::models::BranchCategory> = (0..model_exp.row_count())
            .filter_map(|i| model_exp.row_data(i))
            .map(|r| crate::models::BranchCategory::new(lib_id, r.name_ru.to_string(), r.name_en.to_string()))
            .collect();
        if let Some(path) = rfd::FileDialog::new().add_filter("JSON", &["json"]).save_file() {
            if let Err(e) = export_branch_categories_to_path(path.as_path(), &categories) {
                eprintln!("[ERROR] Export branch categories: {}", e);
            }
        }
    });
    let weak_imp = weak_editor.clone();
    let model_imp = model.clone();
    editor.on_import_categories(move || {
        if let Some(path) = rfd::FileDialog::new().add_filter("JSON", &["json"]).pick_file() {
            match import_branch_categories_from_path(path.as_path()) {
                Ok(imported) => {
                    while model_imp.row_count() > 0 {
                        model_imp.remove(0);
                    }
                    for e in imported {
                        model_imp.insert(model_imp.row_count(), CategoryRow {
                            id: -1,
                            name_ru: e.name_ru.into(),
                            name_en: e.name_en.into(),
                        });
                    }
                    if let Some(ed) = weak_imp.upgrade() {
                        ed.set_current_index(if model_imp.row_count() > 0 { 0 } else { -1 });
                        if model_imp.row_count() > 0 {
                            if let Some(r) = model_imp.row_data(0) {
                                ed.set_current_name_ru(r.name_ru.clone());
                                ed.set_current_name_en(r.name_en.clone());
                            }
                        }
                    }
                }
                Err(e) => eprintln!("[ERROR] Import branch categories: {}", e),
            }
        }
    });
    let state_copy = state.clone();
    let weak_copy = weak_editor.clone();
    let model_copy = model.clone();
    let source_ids = source_library_ids.clone();
    editor.on_copy_from_library(move || {
        let Some(ed) = weak_copy.upgrade() else { return };
        let idx = ed.get_copy_source_index();
        if idx < 0 || (idx as usize) >= source_ids.len() {
            return;
        }
        let source_id = source_ids[idx as usize];
        let st = state_copy.borrow();
        if let Some(ref db) = st.database {
            let cat_repo = BranchCategoryRepo::new(db.conn());
            if let Err(e) = copy_branch_categories_between_libraries(&cat_repo, source_id, lib_id) {
                eprintln!("[ERROR] Copy branch categories: {}", e);
                return;
            }
            drop(st);
            let st2 = state_copy.borrow();
            if let Some(ref db2) = st2.database {
                let cat_repo2 = BranchCategoryRepo::new(db2.conn());
                if let Ok(new_cats) = cat_repo2.list_by_library(lib_id) {
                    while model_copy.row_count() > 0 {
                        model_copy.remove(0);
                    }
                    for c in new_cats {
                        model_copy.insert(model_copy.row_count(), CategoryRow {
                            id: c.id.unwrap_or(-1) as i32,
                            name_ru: c.name_ru.into(),
                            name_en: c.name_en.into(),
                        });
                    }
                    ed.set_current_index(if model_copy.row_count() > 0 { 0 } else { -1 });
                    if model_copy.row_count() > 0 {
                        if let Some(r) = model_copy.row_data(0) {
                            ed.set_current_name_ru(r.name_ru.clone());
                            ed.set_current_name_en(r.name_en.clone());
                        }
                    }
                }
            }
        }
    });
    editor.show().unwrap_or_default();
}

/// Open the Formation levels editor window for the given library.
fn show_formation_levels_editor(state: Rc<RefCell<AppState>>, lib_id: i64, lib_name: &str, lang: &str) {
    use slint::ComponentHandle;
    let (levels, other_library_items, source_library_ids) = {
        let st = state.borrow();
        let db = match st.database.as_ref() {
            Some(d) => d,
            None => {
                eprintln!("[ERROR] Database not initialized");
                return;
            }
        };
        let level_repo = FormationLevelRepo::new(db.conn());
        let levels = match level_repo.list_by_library(lib_id) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[ERROR] Failed to load formation levels: {}", e);
                return;
            }
        };
        let lib_repo = crate::db::repositories::LibraryRepo::new(db.conn());
        let all_libs = match lib_repo.list_all() {
            Ok(l) => l,
            Err(_) => vec![],
        };
        let mut other_items = Vec::new();
        let mut source_ids = Vec::new();
        for l in all_libs {
            if l.id != Some(lib_id) {
                if let Some(id) = l.id {
                    other_items.push(OtherLibraryItem { id: id as i32, name: l.name.into() });
                    source_ids.push(id);
                }
            }
        }
        (levels, other_items, source_ids)
    };
    let rows: Vec<FormationLevelRow> = levels
        .into_iter()
        .map(|l| FormationLevelRow {
            id: l.id.unwrap_or(-1) as i32,
            name_ru: l.name_ru.into(),
            name_en: l.name_en.into(),
            standard_level_ordinal: l.standard_level_ordinal,
        })
        .collect();
    let standard_names: Vec<SharedString> = StandardFormationLevel::all()
        .iter()
        .map(|s| s.name_en().into())
        .collect();
    let editor = match FormationLevelsEditor::new() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("[ERROR] Failed to create Formation levels editor: {}", e);
            return;
        }
    };
    editor.set_library_id(lib_id as i32);
    editor.set_library_name(lib_name.into());
    editor.set_standard_level_names(ModelRc::new(VecModel::from(standard_names)));
    let model = Rc::new(VecModel::from(rows));
    editor.set_custom_levels(ModelRc::new(model.clone()));
    editor.set_current_index(-1);
    editor.set_current_name_ru(Default::default());
    editor.set_current_name_en(Default::default());
    editor.set_current_standard_ordinal(0);
    editor.set_tr_formation_levels_title(ui_tr(lang, "Formation levels").into());
    editor.set_tr_name_russian(ui_tr(lang, "Name (Russian)").into());
    editor.set_tr_name_english(ui_tr(lang, "Name (English)").into());
    editor.set_tr_corresponds_to(ui_tr(lang, "Corresponds to").into());
    editor.set_tr_add_level(ui_tr(lang, "Add level").into());
    editor.set_tr_delete_level(ui_tr(lang, "Delete level").into());
    editor.set_tr_export(ui_tr(lang, "Export…").into());
    editor.set_tr_import(ui_tr(lang, "Import…").into());
    editor.set_tr_copy_from_library(ui_tr(lang, "Copy from library").into());
    editor.set_tr_close(ui_tr(lang, "Close").into());
    editor.set_other_libraries(ModelRc::new(VecModel::from(other_library_items)));
    editor.set_copy_source_index(-1);
    let state_close = state.clone();
    let weak_editor = editor.as_weak();
    let weak_add = weak_editor.clone();
    let model_add = model.clone();
    editor.on_add_level(move || {
        let Some(ed) = weak_add.upgrade() else { return };
        let row = FormationLevelRow {
            id: -1,
            name_ru: Default::default(),
            name_en: Default::default(),
            standard_level_ordinal: 0,
        };
        model_add.insert(model_add.row_count(), row.clone());
        ed.set_current_index(model_add.row_count() as i32 - 1);
        ed.set_current_name_ru(row.name_ru.clone());
        ed.set_current_name_en(row.name_en.clone());
        ed.set_current_standard_ordinal(row.standard_level_ordinal);
    });
    let weak_del = weak_editor.clone();
    let model_del = model.clone();
    editor.on_delete_level(move || {
        let Some(ed) = weak_del.upgrade() else { return };
        let idx = ed.get_current_index();
        if idx >= 0 && (idx as usize) < model_del.row_count() {
            model_del.remove(idx as usize);
            let new_count = model_del.row_count();
            if new_count == 0 {
                ed.set_current_index(-1);
                ed.set_current_name_ru(Default::default());
                ed.set_current_name_en(Default::default());
                ed.set_current_standard_ordinal(0);
            } else {
                let new_idx = (idx as usize).min(new_count - 1);
                ed.set_current_index(new_idx as i32);
                if let Some(r) = model_del.row_data(new_idx) {
                    ed.set_current_name_ru(r.name_ru.clone());
                    ed.set_current_name_en(r.name_en.clone());
                    ed.set_current_standard_ordinal(r.standard_level_ordinal);
                }
            }
        }
    });
    let weak_close = weak_editor.clone();
    let model_close = model.clone();
    editor.on_close_editor(move || {
        let Some(ed) = weak_close.upgrade() else { return };
        let idx = ed.get_current_index();
        if idx >= 0 && (idx as usize) < model_close.row_count() {
            let ru = ed.get_current_name_ru();
            let en = ed.get_current_name_en();
            let ord = ed.get_current_standard_ordinal();
            if let Some(r) = model_close.row_data(idx as usize) {
                let _ = model_close.set_row_data(idx as usize, FormationLevelRow {
                    id: r.id,
                    name_ru: ru,
                    name_en: en,
                    standard_level_ordinal: ord,
                });
            }
        }
        let st = state_close.borrow();
        if let Some(ref db) = st.database {
            let repo = FormationLevelRepo::new(db.conn());
            let existing = match repo.list_by_library(lib_id) {
                Ok(l) => l,
                Err(_) => vec![],
            };
            for old in &existing {
                if let Some(id) = old.id {
                    let _ = repo.delete(id);
                }
            }
            for i in 0..model_close.row_count() {
                if let Some(r) = model_close.row_data(i) {
                    let mut custom = CustomFormationLevel::new(
                        lib_id,
                        r.name_ru.to_string(),
                        r.name_en.to_string(),
                        r.standard_level_ordinal,
                    );
                    let _ = repo.create(&mut custom);
                }
            }
        }
        let _ = ed.hide();
    });
    let weak_sel = weak_editor.clone();
    let model_sel = model.clone();
    editor.on_selection_changed(move |index| {
        let Some(ed) = weak_sel.upgrade() else { return };
        if index >= 0 && (index as usize) < model_sel.row_count() {
            if let Some(r) = model_sel.row_data(index as usize) {
                ed.set_current_name_ru(r.name_ru.clone());
                ed.set_current_name_en(r.name_en.clone());
                ed.set_current_standard_ordinal(r.standard_level_ordinal);
            }
        }
    });
    let weak_form = weak_editor.clone();
    let model_form = model.clone();
    editor.on_form_changed(move |ru, en, ord| {
        let Some(ed) = weak_form.upgrade() else { return };
        let idx = ed.get_current_index();
        if idx >= 0 && (idx as usize) < model_form.row_count() {
            let id = model_form.row_data(idx as usize).map(|r| r.id).unwrap_or(-1);
            let _ = model_form.set_row_data(idx as usize, FormationLevelRow {
                id,
                name_ru: ru,
                name_en: en,
                standard_level_ordinal: ord,
            });
        }
    });
    let model_exp = model.clone();
    editor.on_export_levels(move || {
        let levels: Vec<CustomFormationLevel> = (0..model_exp.row_count())
            .filter_map(|i| model_exp.row_data(i))
                            .map(|r| CustomFormationLevel::new(lib_id, r.name_ru.to_string(), r.name_en.to_string(), r.standard_level_ordinal))
                            .collect();
        if let Some(path) = rfd::FileDialog::new().add_filter("JSON", &["json"]).save_file() {
            if let Err(e) = export_formation_levels_to_path(path.as_path(), &levels) {
                eprintln!("[ERROR] Export formation levels: {}", e);
            }
        }
    });
    let weak_imp = weak_editor.clone();
    let model_imp = model.clone();
    editor.on_import_levels(move || {
        if let Some(path) = rfd::FileDialog::new().add_filter("JSON", &["json"]).pick_file() {
            match import_formation_levels_from_path(path.as_path()) {
                Ok(imported) => {
                    while model_imp.row_count() > 0 {
                        model_imp.remove(0);
                    }
                    for e in imported {
                        model_imp.insert(model_imp.row_count(), FormationLevelRow {
                            id: -1,
                            name_ru: e.name_ru.into(),
                            name_en: e.name_en.into(),
                            standard_level_ordinal: e.standard_level_ordinal,
                        });
                    }
                    if let Some(ed) = weak_imp.upgrade() {
                        ed.set_current_index(if model_imp.row_count() > 0 { 0 } else { -1 });
                        if model_imp.row_count() > 0 {
                            if let Some(r) = model_imp.row_data(0) {
                                ed.set_current_name_ru(r.name_ru.clone());
                                ed.set_current_name_en(r.name_en.clone());
                                ed.set_current_standard_ordinal(r.standard_level_ordinal);
                            }
                        }
                    }
                }
                Err(e) => eprintln!("[ERROR] Import formation levels: {}", e),
            }
        }
    });
    let state_copy = state.clone();
    let weak_copy = weak_editor.clone();
    let model_copy = model.clone();
    let source_ids = source_library_ids.clone();
    editor.on_copy_from_library(move || {
        let Some(ed) = weak_copy.upgrade() else { return };
        let idx = ed.get_copy_source_index();
        if idx < 0 || (idx as usize) >= source_ids.len() {
            return;
        }
        let source_id = source_ids[idx as usize];
        let st = state_copy.borrow();
        if let Some(ref db) = st.database {
            let level_repo = FormationLevelRepo::new(db.conn());
            if let Err(e) = copy_formation_levels_between_libraries(&level_repo, source_id, lib_id) {
                eprintln!("[ERROR] Copy formation levels: {}", e);
                return;
            }
            drop(st);
            let st2 = state_copy.borrow();
            if let Some(ref db2) = st2.database {
                let level_repo2 = FormationLevelRepo::new(db2.conn());
                if let Ok(new_levels) = level_repo2.list_by_library(lib_id) {
                    while model_copy.row_count() > 0 {
                        model_copy.remove(0);
                    }
                    for l in new_levels {
                        model_copy.insert(model_copy.row_count(), FormationLevelRow {
                            id: l.id.unwrap_or(-1) as i32,
                            name_ru: l.name_ru.into(),
                            name_en: l.name_en.into(),
                            standard_level_ordinal: l.standard_level_ordinal,
                        });
                    }
                    ed.set_current_index(if model_copy.row_count() > 0 { 0 } else { -1 });
                    if model_copy.row_count() > 0 {
                        if let Some(r) = model_copy.row_data(0) {
                            ed.set_current_name_ru(r.name_ru.clone());
                            ed.set_current_name_en(r.name_en.clone());
                            ed.set_current_standard_ordinal(r.standard_level_ordinal);
                        }
                    }
                }
            }
        }
    });
    editor.show().unwrap_or_default();
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
    // Placeholder: one root formation so user can open a tab
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
                use slint::ModelRc;
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
                eprintln!("[INFO] Refreshed libraries list: {} libraries", libraries.len());
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to load libraries: {}", e);
            }
        }
    }
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

#[cfg(test)]
mod tests {
    use crate::i18n::Language;

    #[test]
    fn test_language_switching_callback_setup() {
        // Test that language switching callback can be set up
        // This is a basic smoke test
        let lang = Language::from_code("ru");
        assert_eq!(lang, Language::Russian);
        
        let lang_en = Language::from_code("en");
        assert_eq!(lang_en, Language::English);
        
        let lang_default = Language::from_code("unknown");
        assert_eq!(lang_default, Language::English);
    }

    #[test]
    fn test_language_codes() {
        assert_eq!(Language::English.code(), "en");
        assert_eq!(Language::Russian.code(), "ru");
    }

    #[test]
    fn test_language_names() {
        assert_eq!(Language::English.name(), "English");
        assert_eq!(Language::Russian.name(), "Русский");
    }
}
