//! UI translation functions
//!
//! Translations are loaded from embedded JSON files in the `i18n/` directory.
//! To add or modify translations, edit the corresponding `i18n/<lang>.json` file.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Embedded Russian translation JSON (compiled into the binary).
const RU_JSON: &str = include_str!("../../i18n/ru.json");

/// Lazily parsed Russian translation map.
fn ru_translations() -> &'static HashMap<String, String> {
    static RU: OnceLock<HashMap<String, String>> = OnceLock::new();
    RU.get_or_init(|| {
        serde_json::from_str(RU_JSON).unwrap_or_else(|e| {
            log::error!("Failed to parse ru.json translations: {}", e);
            HashMap::new()
        })
    })
}

/// Return translated string for UI. Keys are English strings (msgid style).
/// For non-Russian languages, returns the key unchanged.
pub(crate) fn ui_tr(lang: &str, key: &str) -> String {
    if lang != "ru" {
        return key.to_string();
    }
    ru_translations()
        .get(key)
        .cloned()
        .unwrap_or_else(|| key.to_string())
}

/// Set all UI string properties from Rust so they update when language changes.
pub(crate) fn apply_ui_translations(window: &super::MainWindow, lang: &str) {
    window.set_tr_new_library(ui_tr(lang, "New Library").into());
    window.set_tr_open_library(ui_tr(lang, "Open Library").into());
    window.set_tr_save_library(ui_tr(lang, "Save Library").into());
    window.set_tr_language(ui_tr(lang, "Language").into());
    window.set_tr_libraries(ui_tr(lang, "Libraries").into());
    window.set_tr_welcome_title(ui_tr(lang, "Welcome to TOEditor").into());
    window.set_tr_welcome_desc(
        ui_tr(lang, "Create a new library or open an existing one to get started.").into(),
    );
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

#[cfg(test)]
mod tests {
    use super::ui_tr;
    use crate::i18n::Language;

    #[test]
    fn test_language_switching_callback_setup() {
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

    #[test]
    fn test_ui_tr_english_returns_key() {
        assert_eq!(ui_tr("en", "New Library"), "New Library");
        assert_eq!(ui_tr("en", "File"), "File");
        assert_eq!(ui_tr("en", "Close"), "Close");
        assert_eq!(ui_tr("en", "Unknown Key"), "Unknown Key");
    }

    #[test]
    fn test_ui_tr_russian_known_keys() {
        assert_eq!(ui_tr("ru", "New Library"), "Новая библиотека");
        assert_eq!(ui_tr("ru", "File"), "Файл");
        assert_eq!(ui_tr("ru", "Close"), "Закрыть");
        assert_eq!(ui_tr("ru", "Delete"), "Удалить");
        assert_eq!(ui_tr("ru", "Library"), "Библиотека");
        assert_eq!(ui_tr("ru", "Formation levels"), "Уровни формирований");
        assert_eq!(ui_tr("ru", "Branches…"), "Роды войск…");
        assert_eq!(ui_tr("ru", "Branch categories…"), "Категории родов войск…");
        assert_eq!(ui_tr("ru", "Category"), "Категория");
        assert_eq!(ui_tr("ru", "Add"), "Добавить");
    }

    #[test]
    fn test_ui_tr_russian_unknown_key_returns_key() {
        assert_eq!(ui_tr("ru", "This key does not exist"), "This key does not exist");
    }

    #[test]
    fn test_ui_tr_other_language_returns_key() {
        assert_eq!(ui_tr("fr", "New Library"), "New Library");
        assert_eq!(ui_tr("de", "File"), "File");
    }

    #[test]
    fn test_ui_tr_menu_items() {
        assert_eq!(ui_tr("ru", "Export…"), "Экспорт…");
        assert_eq!(ui_tr("ru", "Import…"), "Импорт…");
        assert_eq!(ui_tr("ru", "Copy from library"), "Копировать из библиотеки");
        assert_eq!(ui_tr("ru", "Name (Russian)"), "Название (рус.)");
        assert_eq!(ui_tr("ru", "Name (English)"), "Название (англ.)");
    }

    #[test]
    fn test_ru_translations_loaded() {
        let map = super::ru_translations();
        assert!(map.len() > 90, "Expected 90+ translations, got {}", map.len());
        assert_eq!(map.get("File"), Some(&"Файл".to_string()));
    }
}
