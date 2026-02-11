//! Main application module

slint::include_modules!();

use anyhow::Result;
use slint::{ComponentHandle, ModelRc, VecModel, Weak, SharedString};
use crate::i18n::{Language, TranslationManager};

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

/// Main application window structure
pub struct AppMainWindow {
    window: MainWindow,
    translation_manager: TranslationManager,
    settings: crate::config::Settings,
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
        
        // Set up UI callbacks FIRST, before any other initialization
        eprintln!("[INIT] Setting up callbacks...");
        setup_callbacks(&window, &mut translation_manager, &mut settings)?;
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
        
        Ok(Self { window, translation_manager, settings })
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
fn setup_callbacks(
    window: &MainWindow,
    translation_manager: &mut TranslationManager,
    settings: &mut crate::config::Settings,
) -> Result<()> {
    let weak_window = window.as_weak();
    
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
                let lang = Language::from_code("en");
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
                let lang = Language::from_code("ru");
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
    
    // Stub handlers for all other menu items (TODO: implement functionality)
    window.on_file_new_library(|| { eprintln!("[DEBUG] File > New Library"); });
    window.on_file_open_library(|| { eprintln!("[DEBUG] File > Open Library"); });
    window.on_file_recent_libraries(|| { eprintln!("[DEBUG] File > Recent Libraries"); });
    window.on_file_save_library(|| { eprintln!("[DEBUG] File > Save Library"); });
    window.on_file_save_library_as(|| { eprintln!("[DEBUG] File > Save Library As"); });
    window.on_file_import_library(|| { eprintln!("[DEBUG] File > Import Library"); });
    window.on_file_import_formation(|| { eprintln!("[DEBUG] File > Import Formation"); });
    window.on_file_export_library(|| { eprintln!("[DEBUG] File > Export Library"); });
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
    window.on_library_properties(|| { eprintln!("[DEBUG] Library > Properties"); });
    window.on_library_manage_tags(|| { eprintln!("[DEBUG] Library > Manage Tags"); });
    window.on_library_export_library(|| { eprintln!("[DEBUG] Library > Export Library"); });
    window.on_library_view_history(|| { eprintln!("[DEBUG] Library > View History"); });
    window.on_library_create_snapshot(|| { eprintln!("[DEBUG] Library > Create Snapshot"); });
    window.on_library_compare_versions(|| { eprintln!("[DEBUG] Library > Compare Versions"); });
    window.on_library_revert_to_version(|| { eprintln!("[DEBUG] Library > Revert to Version"); });
    
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
    window.on_view_color_scheme(|| { eprintln!("[DEBUG] View > Color Scheme"); });
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
    use super::*;
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
