use std::collections::BTreeSet;

use eframe::egui;

use crate::object_catalog::{self, ObjectCatalogKind, ObjectCategory};

/// Reusable object library picker widget.
pub struct ObjectLibraryWidget<'a> {
    search_query: &'a mut String,
    selected_object_kind: &'a mut ObjectCatalogKind,
    open_categories: &'a mut BTreeSet<ObjectCategory>,
}

impl<'a> ObjectLibraryWidget<'a> {
    /// Creates a new object library picker widget.
    pub fn new(
        search_query: &'a mut String,
        selected_object_kind: &'a mut ObjectCatalogKind,
        open_categories: &'a mut BTreeSet<ObjectCategory>,
    ) -> Self {
        Self {
            search_query,
            selected_object_kind,
            open_categories,
        }
    }
}

impl egui::Widget for ObjectLibraryWidget<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut changed = false;

        let mut response = ui
            .vertical(|ui| {
                if let Some(entry) = object_catalog::find_entry(*self.selected_object_kind) {
                    let _ = ui.label(format!(
                        "Selected: {} {}",
                        entry.name,
                        entry.category.icon()
                    ));
                }

                let _ = ui.horizontal(|ui| {
                    let _ = ui.label("🔍");
                    let _ = ui.text_edit_singleline(self.search_query);
                });
                let _ = ui.separator();

                let _ = egui::ScrollArea::vertical().show(ui, |ui| {
                    let query = self.search_query.to_lowercase();
                    for &category in object_catalog::CATEGORY_ORDER {
                        let matching_entries: Vec<_> = object_catalog::ALL_OBJECTS
                            .iter()
                            .copied()
                            .filter(|entry| {
                                entry.category == category
                                    && (query.is_empty()
                                        || entry.name.to_lowercase().contains(&query))
                            })
                            .collect();

                        if matching_entries.is_empty() {
                            continue;
                        }

                        let category_title = format!("{} {}", category.icon(), category.label());

                        if query.is_empty() {
                            let is_open = self.open_categories.contains(&category);
                            let expand_label = if is_open {
                                format!("[-] {category_title}")
                            } else {
                                format!("[+] {category_title}")
                            };

                            if ui.button(expand_label).clicked() {
                                if is_open {
                                    let _ = self.open_categories.remove(&category);
                                } else {
                                    let _ = self.open_categories.insert(category);
                                }
                            }

                            if self.open_categories.contains(&category) {
                                for entry in matching_entries {
                                    let item_response = ui.selectable_value(
                                        self.selected_object_kind,
                                        entry.kind,
                                        entry.name,
                                    );
                                    changed |= item_response.changed();
                                }
                                let _ = ui.add_space(4.0);
                            }
                        } else {
                            let _ = ui.label(egui::RichText::new(category_title).strong());
                            for entry in matching_entries {
                                let item_response = ui.selectable_value(
                                    self.selected_object_kind,
                                    entry.kind,
                                    entry.name,
                                );
                                changed |= item_response.changed();
                            }
                            let _ = ui.add_space(4.0);
                        }
                    }
                });
            })
            .response;

        if changed {
            response.mark_changed();
        }

        response
    }
}
