//! Formation levels editor window

use std::rc::Rc;
use std::cell::RefCell;
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};

use crate::models::{StandardFormationLevel, CustomFormationLevel};
use crate::db::repositories::FormationLevelRepo;
use crate::export::{
    export_formation_levels_to_path, import_formation_levels_from_path,
    copy_formation_levels_between_libraries,
};

use super::super::{FormationLevelsEditor, FormationLevelRow, OtherLibraryItem, AppState};
use super::super::translations::ui_tr;

/// Open the Formation levels editor window for the given library.
pub(in crate::app) fn show_formation_levels_editor(
    state: Rc<RefCell<AppState>>,
    lib_id: i64,
    lib_name: &str,
    lang: &str,
) {
    let (levels, other_library_items, source_library_ids) = {
        let st = state.borrow();
        let db = match st.database.as_ref() {
            Some(d) => d,
            None => {
                log::error!("Database not initialized");
                return;
            }
        };
        let level_repo = FormationLevelRepo::new(db.conn());
        let levels = match level_repo.list_by_library(lib_id) {
            Ok(l) => l,
            Err(e) => {
                log::error!("Failed to load formation levels: {}", e);
                return;
            }
        };
        let lib_repo = crate::db::repositories::LibraryRepo::new(db.conn());
        let all_libs = lib_repo.list_all().unwrap_or_default();
        let mut other_items = Vec::new();
        let mut source_ids = Vec::new();
        for l in all_libs {
            if l.id != Some(lib_id) {
                if let Some(id) = l.id {
                    other_items.push(OtherLibraryItem {
                        id: id as i32,
                        name: l.name.into(),
                    });
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
            log::error!("Failed to create Formation levels editor: {}", e);
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
        let Some(ed) = weak_add.upgrade() else {
            return;
        };
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
        let Some(ed) = weak_del.upgrade() else {
            return;
        };
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
        let Some(ed) = weak_close.upgrade() else {
            return;
        };
        let idx = ed.get_current_index();
        if idx >= 0 && (idx as usize) < model_close.row_count() {
            let ru = ed.get_current_name_ru();
            let en = ed.get_current_name_en();
            let ord = ed.get_current_standard_ordinal();
            if let Some(r) = model_close.row_data(idx as usize) {
                model_close.set_row_data(
                    idx as usize,
                    FormationLevelRow {
                        id: r.id,
                        name_ru: ru,
                        name_en: en,
                        standard_level_ordinal: ord,
                    },
                );
            }
        }
        let st = state_close.borrow();
        if let Some(ref db) = st.database {
            let conn = db.conn();
            if let Err(e) = conn.execute_batch("BEGIN IMMEDIATE") {
                log::error!("Failed to begin transaction: {}", e);
            } else {
                let repo = FormationLevelRepo::new(conn);
                let mut ok = true;
                let existing = repo.list_by_library(lib_id).unwrap_or_default();
                for old in &existing {
                    if let Some(id) = old.id {
                        if repo.delete(id).is_err() {
                            ok = false;
                            break;
                        }
                    }
                }
                if ok {
                    for i in 0..model_close.row_count() {
                        if let Some(r) = model_close.row_data(i) {
                            let mut custom = CustomFormationLevel::new(
                                lib_id,
                                r.name_ru.to_string(),
                                r.name_en.to_string(),
                                r.standard_level_ordinal,
                            );
                            if repo.create(&mut custom).is_err() {
                                ok = false;
                                break;
                            }
                        }
                    }
                }
                if ok {
                    let _ = conn.execute_batch("COMMIT");
                } else {
                    log::error!("Rolling back formation levels save for library {}", lib_id);
                    let _ = conn.execute_batch("ROLLBACK");
                }
            }
        }
        let _ = ed.hide();
    });
    let weak_sel = weak_editor.clone();
    let model_sel = model.clone();
    editor.on_selection_changed(move |index| {
        let Some(ed) = weak_sel.upgrade() else {
            return;
        };
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
        let Some(ed) = weak_form.upgrade() else {
            return;
        };
        let idx = ed.get_current_index();
        if idx >= 0 && (idx as usize) < model_form.row_count() {
            let id = model_form
                .row_data(idx as usize)
                .map(|r| r.id)
                .unwrap_or(-1);
            model_form.set_row_data(
                idx as usize,
                FormationLevelRow {
                    id,
                    name_ru: ru,
                    name_en: en,
                    standard_level_ordinal: ord,
                },
            );
        }
    });
    let model_exp = model.clone();
    editor.on_export_levels(move || {
        let levels: Vec<CustomFormationLevel> = (0..model_exp.row_count())
            .filter_map(|i| model_exp.row_data(i))
            .map(|r| {
                CustomFormationLevel::new(
                    lib_id,
                    r.name_ru.to_string(),
                    r.name_en.to_string(),
                    r.standard_level_ordinal,
                )
            })
            .collect();
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .save_file()
        {
            if let Err(e) = export_formation_levels_to_path(path.as_path(), &levels) {
                log::error!("Export formation levels: {}", e);
            }
        }
    });
    let weak_imp = weak_editor.clone();
    let model_imp = model.clone();
    editor.on_import_levels(move || {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .pick_file()
        {
            match import_formation_levels_from_path(path.as_path()) {
                Ok(imported) => {
                    while model_imp.row_count() > 0 {
                        model_imp.remove(0);
                    }
                    for e in imported {
                        model_imp.insert(
                            model_imp.row_count(),
                            FormationLevelRow {
                                id: -1,
                                name_ru: e.name_ru.into(),
                                name_en: e.name_en.into(),
                                standard_level_ordinal: e.standard_level_ordinal,
                            },
                        );
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
                Err(e) => log::error!("Import formation levels: {}", e),
            }
        }
    });
    let state_copy = state.clone();
    let weak_copy = weak_editor.clone();
    let model_copy = model.clone();
    let source_ids = source_library_ids.clone();
    editor.on_copy_from_library(move || {
        let Some(ed) = weak_copy.upgrade() else {
            return;
        };
        let idx = ed.get_copy_source_index();
        if idx < 0 || (idx as usize) >= source_ids.len() {
            return;
        }
        let source_id = source_ids[idx as usize];
        let st = state_copy.borrow();
        if let Some(ref db) = st.database {
            let level_repo = FormationLevelRepo::new(db.conn());
            if let Err(e) =
                copy_formation_levels_between_libraries(&level_repo, source_id, lib_id)
            {
                log::error!("Copy formation levels: {}", e);
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
                        model_copy.insert(
                            model_copy.row_count(),
                            FormationLevelRow {
                                id: l.id.unwrap_or(-1) as i32,
                                name_ru: l.name_ru.into(),
                                name_en: l.name_en.into(),
                                standard_level_ordinal: l.standard_level_ordinal,
                            },
                        );
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
