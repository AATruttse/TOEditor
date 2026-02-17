//! Branches editor window

use std::rc::Rc;
use std::cell::RefCell;
use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::models::Branch;
use crate::db::repositories::{BranchRepo, BranchCategoryRepo};
use crate::export::{
    export_branches_to_path, import_branches_from_path, copy_branches_between_libraries,
};

use super::super::{BranchesEditor, BranchRow, OtherLibraryItem, CategoryItem, AppState};
use super::super::translations::ui_tr;

/// Open the Branches editor window for the given library.
pub(in crate::app) fn show_branches_editor(
    state: Rc<RefCell<AppState>>,
    lib_id: i64,
    lib_name: &str,
    lang: &str,
) {
    let (branches, other_library_items, source_library_ids, category_items) = {
        let st = state.borrow();
        let db = match st.database.as_ref() {
            Some(d) => d,
            None => {
                log::error!("Database not initialized");
                return;
            }
        };
        let branch_repo = BranchRepo::new(db.conn());
        let mut branches = match branch_repo.list_by_library(lib_id) {
            Ok(b) => b,
            Err(e) => {
                log::error!("Failed to load branches: {}", e);
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
        let categories = BranchCategoryRepo::new(db.conn())
            .list_by_library(lib_id)
            .unwrap_or_default();
        let category_items: Vec<CategoryItem> = categories
            .iter()
            .map(|c| CategoryItem {
                id: c.id.unwrap_or(-1) as i32,
                name: if lang == "ru" {
                    c.name_ru.as_str()
                } else {
                    c.name_en.as_str()
                }
                .into(),
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
            log::error!("Failed to create Branches editor: {}", e);
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
        let Some(ed) = weak_add.upgrade() else {
            return;
        };
        let row = BranchRow {
            id: -1,
            category_id: -1,
            name_ru: Default::default(),
            name_en: Default::default(),
        };
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
                ed.set_current_category_index(-1);
            } else {
                let new_idx = (idx as usize).min(new_count - 1);
                ed.set_current_index(new_idx as i32);
                if let Some(r) = model_del.row_data(new_idx) {
                    ed.set_current_name_ru(r.name_ru.clone());
                    ed.set_current_name_en(r.name_en.clone());
                    let cat_idx = if r.category_id > 0 {
                        category_items_del
                            .iter()
                            .position(|c| c.id == r.category_id)
                            .map(|i| i as i32)
                            .unwrap_or(-1)
                    } else {
                        -1
                    };
                    ed.set_current_category_index(cat_idx);
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
            if let Some(r) = model_close.row_data(idx as usize) {
                model_close.set_row_data(
                    idx as usize,
                    BranchRow {
                        id: r.id,
                        category_id: r.category_id,
                        name_ru: ru,
                        name_en: en,
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
                let repo = BranchRepo::new(conn);
                let mut ok = repo.delete_by_library(lib_id).is_ok();
                if ok {
                    for i in 0..model_close.row_count() {
                        if let Some(r) = model_close.row_data(i) {
                            let cat_id = if r.category_id > 0 {
                                Some(r.category_id as i64)
                            } else {
                                None
                            };
                            let mut b = Branch::with_category(
                                lib_id,
                                cat_id,
                                r.name_ru.to_string(),
                                r.name_en.to_string(),
                            );
                            if repo.create(&mut b).is_err() {
                                ok = false;
                                break;
                            }
                        }
                    }
                }
                if ok {
                    let _ = conn.execute_batch("COMMIT");
                } else {
                    log::error!("Rolling back branches save for library {}", lib_id);
                    let _ = conn.execute_batch("ROLLBACK");
                }
            }
        }
        let _ = ed.hide();
    });
    let weak_sel = weak_editor.clone();
    let model_sel = model.clone();
    let category_items_sel = category_items_clone.clone();
    editor.on_selection_changed(move |index| {
        let Some(ed) = weak_sel.upgrade() else {
            return;
        };
        if index >= 0 && (index as usize) < model_sel.row_count() {
            if let Some(r) = model_sel.row_data(index as usize) {
                ed.set_current_name_ru(r.name_ru.clone());
                ed.set_current_name_en(r.name_en.clone());
                let cat_idx = if r.category_id > 0 {
                    category_items_sel
                        .iter()
                        .position(|c| c.id == r.category_id)
                        .map(|i| i as i32)
                        .unwrap_or(-1)
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
        let Some(ed) = weak_cat.upgrade() else {
            return;
        };
        let idx = ed.get_current_index();
        if idx >= 0
            && (idx as usize) < model_cat.row_count()
            && index >= 0
            && (index as usize) < category_items_cat.len()
        {
            if let Some(r) = model_cat.row_data(idx as usize) {
                let new_cat_id = category_items_cat[index as usize].id;
                model_cat.set_row_data(
                    idx as usize,
                    BranchRow {
                        id: r.id,
                        category_id: new_cat_id,
                        name_ru: r.name_ru.clone(),
                        name_en: r.name_en.clone(),
                    },
                );
                ed.set_current_category_index(index);
            }
        }
    });
    let model_exp = model.clone();
    editor.on_export_branches(move || {
        let branches: Vec<Branch> = (0..model_exp.row_count())
            .filter_map(|i| model_exp.row_data(i))
            .map(|r| {
                Branch::with_category(lib_id, None, r.name_ru.to_string(), r.name_en.to_string())
            })
            .collect();
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .save_file()
        {
            if let Err(e) = export_branches_to_path(path.as_path(), &branches) {
                log::error!("Export branches: {}", e);
            }
        }
    });
    let weak_imp = weak_editor.clone();
    let model_imp = model.clone();
    editor.on_import_branches(move || {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .pick_file()
        {
            match import_branches_from_path(path.as_path()) {
                Ok(imported) => {
                    while model_imp.row_count() > 0 {
                        model_imp.remove(0);
                    }
                    for e in imported {
                        model_imp.insert(
                            model_imp.row_count(),
                            BranchRow {
                                id: -1,
                                category_id: -1,
                                name_ru: e.name_ru.into(),
                                name_en: e.name_en.into(),
                            },
                        );
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
                Err(e) => log::error!("Import branches: {}", e),
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
            let branch_repo = BranchRepo::new(db.conn());
            if let Err(e) = copy_branches_between_libraries(&branch_repo, source_id, lib_id) {
                log::error!("Copy branches: {}", e);
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
                        model_copy.insert(
                            model_copy.row_count(),
                            BranchRow {
                                id: b.id.unwrap_or(-1) as i32,
                                category_id: b.category_id.unwrap_or(-1) as i32,
                                name_ru: b.name_ru.into(),
                                name_en: b.name_en.into(),
                            },
                        );
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
