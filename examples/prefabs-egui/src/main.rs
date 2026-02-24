//! Prefabs egui
#[allow(missing_docs, unused_results)]
use eframe::egui;
use egui_plot::{Plot, PlotImage, PlotPoint, PlotPoints, Points};
use insim_core::{heading::Heading, object::ObjectInfo};
use insim_lyt::Lyt;
use std::path::{Path, PathBuf};

#[allow(missing_docs)]
/// Inspector UI helpers.
mod inspector;
#[allow(missing_docs)]
/// Object library picker widget.
mod object_library_widget;
/// Placeable object catalog.
mod object_catalog;
pub mod tools;

const CLICK_RADIUS_UNITS: i64 = 160;
const DEFAULT_PLACE_Z_UNITS: u8 = 0;
const RAW_UNITS_PER_METRE: f32 = 16.0;
const DEFAULT_LYT_VERSION: u8 = 0;
const DEFAULT_LYT_REVISION: u8 = 252;
const DEFAULT_LYT_LAPS: u8 = 0;
const DEFAULT_LYT_MINI_REV: u8 = 9;

#[derive(Debug, Clone, Copy)]
struct LytHeader {
    version: u8,
    revision: u8,
    laps: u8,
    mini_rev: u8,
}

impl Default for LytHeader {
    fn default() -> Self {
        Self {
            version: DEFAULT_LYT_VERSION,
            revision: DEFAULT_LYT_REVISION,
            laps: DEFAULT_LYT_LAPS,
            mini_rev: DEFAULT_LYT_MINI_REV,
        }
    }
}

struct LoadedBackground {
    texture: egui::TextureHandle,
    size_units: [f32; 2],
    source_size_px: [u32; 2],
}

// --- 1. DATA STRUCTURES ---

struct TrackEditor {
    background: Option<LoadedBackground>,
    background_image_path: Option<PathBuf>,
    lyt_path: Option<PathBuf>,
    lyt_header: LytHeader,
    status_line: Option<String>,
    tools: tools::Tools,
    objects: Vec<ObjectInfo>,
    object_ids: Vec<u64>,
    next_object_id: u64,
    select_drag_start: Option<[i16; 2]>,
    select_drag_current: Option<[i16; 2]>,
    select_drag_additive: bool,
}

impl TrackEditor {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let _ = cc;

        Self {
            objects: Vec::new(),
            object_ids: Vec::new(),
            next_object_id: 0,
            select_drag_start: None,
            select_drag_current: None,
            select_drag_additive: false,
            background: None,
            background_image_path: None,
            lyt_path: None,
            lyt_header: LytHeader::default(),
            status_line: None,
            tools: tools::Tools::default(),
        }
    }

    fn insert_object(&mut self, object: ObjectInfo) -> u64 {
        let object_id = self.next_object_id;
        self.objects.push(object);
        self.object_ids.push(object_id);
        self.next_object_id += 1;
        object_id
    }

    fn clear_select_drag(&mut self) {
        self.select_drag_start = None;
        self.select_drag_current = None;
        self.select_drag_additive = false;
    }

    fn commit_select_drag_selection(&mut self) {
        let (Some(start), Some(end)) = (self.select_drag_start, self.select_drag_current) else {
            self.clear_select_drag();
            return;
        };

        let min_x = start[0].min(end[0]);
        let max_x = start[0].max(end[0]);
        let min_y = start[1].min(end[1]);
        let max_y = start[1].max(end[1]);

        let selected = &mut self.tools.select.selected_object_ids;
        if !self.select_drag_additive {
            selected.clear();
        }

        for (object_id, object) in self.object_ids.iter().copied().zip(self.objects.iter()) {
            let pos = object.position();
            if pos.x >= min_x
                && pos.x <= max_x
                && pos.y >= min_y
                && pos.y <= max_y
                && !selected.contains(&object_id)
            {
                selected.push(object_id);
            }
        }

        self.clear_select_drag();
    }

    fn delete_objects_by_ids(&mut self, ids: &[u64]) -> usize {
        use std::collections::HashSet;

        let targets: HashSet<u64> = ids.iter().copied().collect();
        if targets.is_empty() {
            return 0;
        }

        let mut kept_objects = Vec::with_capacity(self.objects.len());
        let mut kept_ids = Vec::with_capacity(self.object_ids.len());
        let mut removed = 0_usize;

        for (object_id, object) in self.object_ids.drain(..).zip(self.objects.drain(..)) {
            if targets.contains(&object_id) {
                removed += 1;
            } else {
                kept_ids.push(object_id);
                kept_objects.push(object);
            }
        }

        self.object_ids = kept_ids;
        self.objects = kept_objects;
        self.tools
            .select
            .selected_object_ids
            .retain(|id| !targets.contains(id));
        self.tools
            .spline_path
            .generated_object_ids
            .retain(|id| !targets.contains(id));
        removed
    }

    fn delete_selected_objects(&mut self) {
        let selected_ids = std::mem::take(&mut self.tools.select.selected_object_ids);
        let _ = self.delete_objects_by_ids(&selected_ids);
    }

    fn replace_objects(&mut self, objects: Vec<ObjectInfo>) {
        let len = objects.len() as u64;
        self.objects = objects;
        self.object_ids = (0..len).collect();
        self.next_object_id = len;
        self.tools.select.selected_object_ids.clear();
        self.tools.spline_path.generated_object_ids.clear();
    }

    fn clear_spline_generated_objects(&mut self) -> usize {
        let ids = std::mem::take(&mut self.tools.spline_path.generated_object_ids);
        self.delete_objects_by_ids(&ids)
    }

    fn apply_spline_objects(&mut self) {
        let control_points = self.tools.spline_path.control_points.clone();
        if control_points.len() < 2 {
            self.status_line = Some("Spline apply needs at least 2 control points.".to_owned());
            return;
        }

        let spacing_units = self.tools.spline_path.spacing_units;
        if spacing_units <= 0 {
            self.status_line = Some("Spline spacing must be greater than 0.".to_owned());
            return;
        }

        let samples = sample_spline_samples_raw(&control_points, spacing_units);
        if samples.is_empty() {
            self.status_line = Some("Spline apply produced no points.".to_owned());
            return;
        }

        let old_generated = self.tools.spline_path.generated_object_ids.clone();
        let removed = self.delete_objects_by_ids(&old_generated);

        let template = self.tools.spline_path.object_template.clone();
        let heading_offset = template
            .heading()
            .map(|heading| {
                heading.to_radians() - heading_from_vec2(samples[0].tangent).to_radians()
            })
            .unwrap_or(0.0);

        let mut new_generated_ids = Vec::with_capacity(samples.len());
        for sample in samples {
            let mut object = template.clone();
            let position = object.position_mut();
            position.x = sample.pos[0];
            position.y = sample.pos[1];
            if let Some(heading) = object.heading_mut() {
                *heading = Heading::from_radians(
                    heading_from_vec2(sample.tangent).to_radians() + heading_offset,
                );
            }
            let object_id = self.insert_object(object);
            new_generated_ids.push(object_id);
        }

        self.tools.select.selected_object_ids = new_generated_ids.clone();
        self.tools.spline_path.generated_object_ids = new_generated_ids;
        self.status_line = Some(format!(
            "Spline applied: {} object(s) generated, {} cleared.",
            self.tools.spline_path.generated_object_ids.len(),
            removed
        ));
    }

    fn load_background_image(&mut self, ctx: &egui::Context, path: PathBuf) {
        match load_image_to_texture(ctx, &path) {
            Ok(background) => {
                self.background = Some(background);
                self.background_image_path = Some(path.clone());
                if let Some(background) = &self.background {
                    self.status_line = Some(format!(
                        "Loaded background image: {} ({}x{} px -> {}x{} raw units)",
                        path.display(),
                        background.source_size_px[0],
                        background.source_size_px[1],
                        background.size_units[0] as i64,
                        background.size_units[1] as i64,
                    ));
                }
            },
            Err(err) => {
                self.status_line = Some(format!("Failed to load background image: {err}"));
            },
        }
    }

    fn load_lyt(&mut self, path: PathBuf) {
        match Lyt::from_path(&path) {
            Ok(lyt) => {
                self.lyt_header = LytHeader {
                    version: lyt.version,
                    revision: lyt.revision,
                    laps: lyt.laps,
                    mini_rev: lyt.mini_rev,
                };
                let count = lyt.objects.len();
                self.replace_objects(lyt.objects);
                self.lyt_path = Some(path.clone());
                self.status_line = Some(format!("Loaded LYT ({} objects): {}", count, path.display()));
            },
            Err(err) => {
                self.status_line = Some(format!("Failed to load LYT: {err}"));
            },
        }
    }

    fn save_lyt(&mut self, path: PathBuf) {
        let output_path = normalize_lyt_path(path);
        let lyt = Lyt {
            version: self.lyt_header.version,
            revision: self.lyt_header.revision,
            laps: self.lyt_header.laps,
            mini_rev: self.lyt_header.mini_rev,
            objects: self.objects.clone(),
        };

        match std::fs::File::create(&output_path) {
            Ok(file) => match lyt.write(file) {
                Ok(_) => {
                    self.lyt_path = Some(output_path.clone());
                    self.status_line = Some(format!(
                        "Saved LYT ({} objects): {}",
                        self.objects.len(),
                        output_path.display()
                    ));
                },
                Err(err) => {
                    self.status_line = Some(format!("Failed to save LYT: {err}"));
                },
            },
            Err(err) => {
                self.status_line = Some(format!(
                    "Failed to create '{}': {err}",
                    output_path.display()
                ));
            },
        }
    }
}

// --- 2. THE UI UPDATE LOOP ---

impl eframe::App for TrackEditor {
    #[allow(missing_docs, unused_results)]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("file_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.button("Load Background Image...").clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("Image", &["png", "jpg", "jpeg", "bmp", "webp"])
                        .pick_file()
                {
                    self.load_background_image(ctx, path);
                }

                if let Some(path) = &self.background_image_path {
                    ui.label(format!("Background: {}", path.display()));
                } else {
                    ui.label("Background: not loaded");
                }

                ui.separator();

                if ui.button("Load LYT...").clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("Layout", &["lyt"])
                        .pick_file()
                {
                    self.load_lyt(path);
                }

                let save_clicked = ui
                    .add_enabled(self.lyt_path.is_some(), egui::Button::new("Save LYT"))
                    .clicked();
                if save_clicked && let Some(path) = self.lyt_path.clone() {
                    self.save_lyt(path);
                }

                if ui.button("Save LYT As...").clicked() {
                    let mut file_dialog = rfd::FileDialog::new().add_filter("Layout", &["lyt"]);
                    if let Some(path) = &self.lyt_path {
                        if let Some(parent) = path.parent() {
                            file_dialog = file_dialog.set_directory(parent);
                        }
                        if let Some(file_name) = path.file_name().and_then(std::ffi::OsStr::to_str)
                        {
                            file_dialog = file_dialog.set_file_name(file_name);
                        }
                    }

                    if let Some(path) = file_dialog.save_file() {
                        self.save_lyt(path);
                    }
                }

                if let Some(path) = &self.lyt_path {
                    ui.label(format!("LYT: {}", path.display()));
                } else {
                    ui.label("LYT: not loaded");
                }
            });

            if let Some(status_line) = &self.status_line {
                ui.label(status_line);
            }
        });

        if self.tools.active == tools::ToolKind::Select
            && ctx.input(|i| i.key_pressed(egui::Key::Delete))
        {
            self.delete_selected_objects();
        }

        if self.tools.active != tools::ToolKind::Select {
            self.clear_select_drag();
        }

        let mut spline_apply_requested = false;
        let mut spline_clear_generated_requested = false;

        egui::SidePanel::left("tools")
            .resizable(false)
            .exact_width(45.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);

                    // Helper for tool buttons
                    let mut tool_button = |kind: tools::ToolKind, icon: &str, tooltip: &str| {
                        let is_active = self.tools.active == kind;
                        let btn = egui::Button::selectable(is_active, icon);
                        if ui
                            .add_sized([30.0, 30.0], btn)
                            .on_hover_text(tooltip)
                            .clicked()
                        {
                            self.tools.activate(kind);
                        }
                        ui.add_space(5.0);
                    };

                    tool_button(tools::ToolKind::Select, "🖱", "Select Object (V)");
                    tool_button(tools::ToolKind::Place, "📦", "Place Object (B)");
                    tool_button(tools::ToolKind::SplinePath, "〰", "Draw Spline (P)");
                    tool_button(tools::ToolKind::RampGen, "📐", "Ramp Generator (R)");
                });
            });

        egui::SidePanel::left("context_panel")
            .exact_width(220.0)
            .show(ctx, |ui| {
                ui.add_space(10.0);

                match self.tools.active {
                    tools::ToolKind::Select => {
                        inspector::show_selection_inspector(
                            ui,
                            &self.object_ids,
                            &mut self.objects,
                            &self.tools.select.selected_object_ids,
                        );
                    },
                    tools::ToolKind::Place => {
                        let place = &mut self.tools.place;
                        ui.heading("Object Library");
                        let _ = ui.add(object_library_widget::ObjectLibraryWidget::new(
                            &mut place.search_query,
                            &mut place.selected_object_kind,
                            &mut place.open_categories,
                        ));
                    },
                    tools::ToolKind::SplinePath => {
                        ui.heading("Catmull-Rom Spline");
                        let spline = &mut self.tools.spline_path;

                        ui.label("Click the map to place control points.");
                        ui.label(format!("Control points: {}", spline.control_points.len()));
                        ui.label(format!(
                            "Generated objects: {}",
                            spline.generated_object_ids.len()
                        ));

                        let _ = ui.add(
                            egui::DragValue::new(&mut spline.spacing_units)
                                .range(1..=5000)
                                .speed(1)
                                .prefix("Spacing: ")
                                .suffix(" units"),
                        );

                        ui.separator();
                        ui.label("Object Library");
                        let kind_changed = ui
                            .add(object_library_widget::ObjectLibraryWidget::new(
                                &mut spline.search_query,
                                &mut spline.selected_object_kind,
                                &mut spline.open_categories,
                            ))
                            .changed();
                        if kind_changed {
                            spline.object_template = spline.selected_object_kind.create_default();
                        }

                        ui.separator();
                        ui.label("Object Template");
                        let _ = ui.add(
                            inspector::ObjectEditorWidget::new(&mut spline.object_template)
                                .options(inspector::ObjectEditorOptions::template()),
                        );

                        ui.separator();
                        if ui.button("Undo Last Point").clicked() {
                            let _ = spline.control_points.pop();
                        }
                        if ui.button("Clear Points").clicked() {
                            spline.control_points.clear();
                            spline.generated_object_ids.clear();
                        }
                        if ui
                            .add_enabled(
                                !spline.generated_object_ids.is_empty(),
                                egui::Button::new("Clear Generated"),
                            )
                            .clicked()
                        {
                            spline_clear_generated_requested = true;
                        }
                        if ui
                            .add_enabled(
                                spline.control_points.len() >= 2,
                                egui::Button::new("Apply"),
                            )
                            .clicked()
                        {
                            spline_apply_requested = true;
                        }
                    },
                    tools::ToolKind::RampGen => {
                        ui.heading("Ramp Generator");
                        ui.label("Select Start/End to build a ramp.");
                    },
                }
            });

        if spline_clear_generated_requested {
            let removed = self.clear_spline_generated_objects();
            self.status_line = Some(format!("Spline generated objects cleared: {}", removed));
        }

        if spline_apply_requested {
            self.apply_spline_objects();
        }

        // -- CENTRAL PANEL: THE 2D MAP --
        let _ = egui::CentralPanel::default().show(ctx, |ui| {
            let plot = Plot::new("track_map")
                .data_aspect(1.0)
                .show_x(false)
                .show_y(false);

            debug_assert_eq!(self.objects.len(), self.object_ids.len());
            let selected_lookup: std::collections::HashSet<u64> = self
                .tools
                .select
                .selected_object_ids
                .iter()
                .copied()
                .collect();

            let spline_control_points = self.tools.spline_path.control_points.clone();
            let spline_curve = sample_catmull_rom_polyline(&spline_control_points);
            let spline_preview_samples =
                sample_spline_samples_raw(&spline_control_points, self.tools.spline_path.spacing_units);

            // 1. Create a variable outside the closure to hold the coordinate
            let mut pointer_coord = None;

            let plot_response = plot.show(ui, |plot_ui| {
                // 2. Grab the coordinate INSIDE the closure using plot_ui
                pointer_coord = plot_ui.pointer_coordinate();

                // Draw Background Map
                if let Some(background) = &self.background {
                    let bg_image = PlotImage::new(
                        "Background",
                        background.texture.id(),
                        PlotPoint::new(0.0, 0.0),
                        background.size_units,
                    );
                    plot_ui.image(bg_image);
                }

                // Draw the Objects
                let mut normal_points = Vec::new();
                let mut selected_point = Vec::new();

                for (object_id, object) in self.object_ids.iter().copied().zip(self.objects.iter())
                {
                    let (x, y) = object_xy_units(object);
                    if selected_lookup.contains(&object_id) {
                        selected_point.push([x, y]);
                    } else {
                        normal_points.push([x, y]);
                    }
                }

                plot_ui.points(
                    Points::new("Objects", PlotPoints::new(normal_points))
                        .radius(4.0)
                        .color(egui::Color32::LIGHT_GRAY)
                        .name("Objects"),
                );

                plot_ui.points(
                    Points::new("Selected", PlotPoints::new(selected_point))
                        .radius(6.0)
                        .color(egui::Color32::YELLOW)
                        .name("Selected"),
                );

                if let (Some(start), Some(current)) = (self.select_drag_start, self.select_drag_current)
                {
                    let sx = f64::from(start[0]);
                    let sy = f64::from(start[1]);
                    let cx = f64::from(current[0]);
                    let cy = f64::from(current[1]);
                    let rect_points = vec![[sx, sy], [cx, sy], [cx, cy], [sx, cy], [sx, sy]];
                    plot_ui.line(
                        egui_plot::Line::new("Select Box", PlotPoints::new(rect_points))
                            .color(egui::Color32::from_rgb(120, 200, 255)),
                    );
                }

                if !spline_curve.is_empty() {
                    plot_ui.line(
                        egui_plot::Line::new("Spline", PlotPoints::new(spline_curve.clone()))
                            .color(egui::Color32::from_rgb(80, 180, 255)),
                    );
                }

                if !spline_control_points.is_empty() {
                    let control_plot_points: Vec<[f64; 2]> = spline_control_points
                        .iter()
                        .map(|point| [f64::from(point[0]), f64::from(point[1])])
                        .collect();
                    plot_ui.points(
                        Points::new("Spline Control", PlotPoints::new(control_plot_points))
                            .radius(5.0)
                            .color(egui::Color32::from_rgb(0, 200, 255)),
                    );
                }

                if !spline_preview_samples.is_empty() {
                    let preview_plot_points: Vec<[f64; 2]> = spline_preview_samples
                        .iter()
                        .map(|sample| [f64::from(sample.pos[0]), f64::from(sample.pos[1])])
                        .collect();
                    plot_ui.points(
                        Points::new("Spline Preview", PlotPoints::new(preview_plot_points))
                            .radius(3.5)
                            .color(egui::Color32::from_rgb(255, 160, 50)),
                    );
                }
            });

            let mut consumed_drag_select = false;

            if self.tools.active == tools::ToolKind::Select {
                let command_drag = ctx.input(|i| i.modifiers.command);
                if plot_response.response.drag_started() && command_drag {
                    if let Some(pointer_pos) = pointer_coord {
                        let drag_start = [
                            plot_to_raw_units(pointer_pos.x),
                            plot_to_raw_units(pointer_pos.y),
                        ];
                        self.select_drag_start = Some(drag_start);
                        self.select_drag_current = Some(drag_start);
                        self.select_drag_additive = true;
                    }
                }

                if self.select_drag_start.is_some() && plot_response.response.dragged() {
                    if let Some(pointer_pos) = pointer_coord {
                        self.select_drag_current = Some([
                            plot_to_raw_units(pointer_pos.x),
                            plot_to_raw_units(pointer_pos.y),
                        ]);
                    }
                }

                if self.select_drag_start.is_some() && plot_response.response.drag_stopped() {
                    self.commit_select_drag_selection();
                    consumed_drag_select = true;
                }
            }

            // 3. Handle Interaction using the variable we captured
            if !consumed_drag_select && plot_response.response.clicked() {
                if let Some(pointer_pos) = pointer_coord {
                    let click_x_raw = plot_to_raw_units(pointer_pos.x);
                    let click_y_raw = plot_to_raw_units(pointer_pos.y);

                    let click_radius_sq = CLICK_RADIUS_UNITS * CLICK_RADIUS_UNITS;

                    let mut closest_object_id = None;
                    let mut closest_dist_sq = i64::MAX;

                    for (object_id, object) in
                        self.object_ids.iter().copied().zip(self.objects.iter())
                    {
                        let pos = object.position();
                        let dx = i64::from(pos.x) - i64::from(click_x_raw);
                        let dy = i64::from(pos.y) - i64::from(click_y_raw);
                        let dist_sq = dx * dx + dy * dy;

                        if dist_sq < click_radius_sq && dist_sq < closest_dist_sq {
                            closest_dist_sq = dist_sq;
                            closest_object_id = Some(object_id);
                        }
                    }

                    match self.tools.active {
                        tools::ToolKind::Select => {
                            let additive = ctx.input(|i| i.modifiers.command || i.modifiers.shift);
                            if let Some(object_id) = closest_object_id {
                                let selected = &mut self.tools.select.selected_object_ids;
                                if additive {
                                    if let Some(existing) =
                                        selected.iter().position(|selected| *selected == object_id)
                                    {
                                        selected.remove(existing);
                                    } else {
                                        selected.push(object_id);
                                    }
                                } else {
                                    selected.clear();
                                    selected.push(object_id);
                                }
                            } else if !additive {
                                self.tools.select.selected_object_ids.clear();
                            }
                        },
                        tools::ToolKind::Place => {
                            let object = make_object_for_kind(
                                self.tools.place.selected_object_kind,
                                click_x_raw,
                                click_y_raw,
                                DEFAULT_PLACE_Z_UNITS,
                            );
                            let object_id = self.insert_object(object);
                            self.tools.select.selected_object_ids.clear();
                            self.tools.select.selected_object_ids.push(object_id);
                            self.tools.activate(tools::ToolKind::Select);
                        },
                        tools::ToolKind::SplinePath => {
                            self.tools
                                .spline_path
                                .control_points
                                .push([click_x_raw, click_y_raw]);
                        },
                        tools::ToolKind::RampGen => {},
                    }
                }
            }
        });
    }
}

// --- 3. UTILITY FUNCTIONS ---

#[derive(Clone, Copy)]
struct SplineSampleRaw {
    pos: [i16; 2],
    tangent: [f64; 2],
}

fn sample_spline_samples_raw(control_points: &[[i16; 2]], spacing_units: i32) -> Vec<SplineSampleRaw> {
    if control_points.len() < 2 || spacing_units <= 0 {
        return Vec::new();
    }

    let curve = sample_catmull_rom_polyline(control_points);
    if curve.len() < 2 {
        return Vec::new();
    }

    let sampled = resample_polyline(&curve, f64::from(spacing_units));
    let mut out = Vec::with_capacity(sampled.len());
    let mut tangent_fallback = [0.0, 1.0];
    for (idx, point) in sampled.iter().copied().enumerate() {
        let prev = if idx > 0 { sampled[idx - 1] } else { point };
        let next = if idx + 1 < sampled.len() {
            sampled[idx + 1]
        } else {
            point
        };
        let tangent = normalize_vec2([next[0] - prev[0], next[1] - prev[1]], tangent_fallback);

        let x = point[0].round().clamp(i16::MIN as f64, i16::MAX as f64) as i16;
        let y = point[1].round().clamp(i16::MIN as f64, i16::MAX as f64) as i16;
        let raw = [x, y];
        if out
            .last()
            .map(|sample: &SplineSampleRaw| sample.pos == raw)
            .unwrap_or(false)
        {
            tangent_fallback = tangent;
            continue;
        }

        out.push(SplineSampleRaw { pos: raw, tangent });
        tangent_fallback = tangent;
    }
    out
}

fn normalize_vec2(vector: [f64; 2], fallback: [f64; 2]) -> [f64; 2] {
    let len_sq = vector[0] * vector[0] + vector[1] * vector[1];
    if len_sq <= f64::EPSILON {
        return fallback;
    }
    let len = len_sq.sqrt();
    [vector[0] / len, vector[1] / len]
}

fn heading_from_vec2(vector: [f64; 2]) -> Heading {
    Heading::from_radians((-vector[0]).atan2(vector[1]))
}

fn sample_catmull_rom_polyline(control_points: &[[i16; 2]]) -> Vec<[f64; 2]> {
    if control_points.len() < 2 {
        return Vec::new();
    }

    if control_points.len() == 2 {
        return vec![
            [f64::from(control_points[0][0]), f64::from(control_points[0][1])],
            [f64::from(control_points[1][0]), f64::from(control_points[1][1])],
        ];
    }

    let points: Vec<[f64; 2]> = control_points
        .iter()
        .map(|point| [f64::from(point[0]), f64::from(point[1])])
        .collect();

    let mut out = Vec::new();
    for i in 0..(points.len() - 1) {
        let p0 = if i == 0 { points[i] } else { points[i - 1] };
        let p1 = points[i];
        let p2 = points[i + 1];
        let p3 = if i + 2 < points.len() {
            points[i + 2]
        } else {
            points[i + 1]
        };

        let seg_dx = p2[0] - p1[0];
        let seg_dy = p2[1] - p1[1];
        let seg_len = (seg_dx * seg_dx + seg_dy * seg_dy).sqrt();
        let samples = ((seg_len / 16.0).ceil() as usize).clamp(8, 128);

        for step in 0..samples {
            let t = step as f64 / samples as f64;
            out.push(catmull_rom(p0, p1, p2, p3, t));
        }
    }

    if let Some(last) = points.last().copied() {
        out.push(last);
    }

    out
}

fn catmull_rom(p0: [f64; 2], p1: [f64; 2], p2: [f64; 2], p3: [f64; 2], t: f64) -> [f64; 2] {
    let t2 = t * t;
    let t3 = t2 * t;

    let x = 0.5
        * ((2.0 * p1[0])
            + (-p0[0] + p2[0]) * t
            + (2.0 * p0[0] - 5.0 * p1[0] + 4.0 * p2[0] - p3[0]) * t2
            + (-p0[0] + 3.0 * p1[0] - 3.0 * p2[0] + p3[0]) * t3);
    let y = 0.5
        * ((2.0 * p1[1])
            + (-p0[1] + p2[1]) * t
            + (2.0 * p0[1] - 5.0 * p1[1] + 4.0 * p2[1] - p3[1]) * t2
            + (-p0[1] + 3.0 * p1[1] - 3.0 * p2[1] + p3[1]) * t3);

    [x, y]
}

fn resample_polyline(polyline: &[[f64; 2]], spacing: f64) -> Vec<[f64; 2]> {
    if polyline.is_empty() || spacing <= 0.0 {
        return Vec::new();
    }

    let mut out = vec![polyline[0]];
    let mut distance_since_last = 0.0;
    let mut previous = polyline[0];

    for &current in &polyline[1..] {
        let mut segment_start = previous;
        let mut dx = current[0] - segment_start[0];
        let mut dy = current[1] - segment_start[1];
        let mut segment_length = (dx * dx + dy * dy).sqrt();

        while segment_length > 0.0 && distance_since_last + segment_length >= spacing {
            let remaining = spacing - distance_since_last;
            let ratio = remaining / segment_length;
            let point = [segment_start[0] + dx * ratio, segment_start[1] + dy * ratio];
            out.push(point);

            segment_start = point;
            dx = current[0] - segment_start[0];
            dy = current[1] - segment_start[1];
            segment_length = (dx * dx + dy * dy).sqrt();
            distance_since_last = 0.0;
        }

        distance_since_last += segment_length;
        previous = current;
    }

    if let Some(last) = polyline.last().copied() {
        let should_push_last = out
            .last()
            .map(|point| {
                let dx = point[0] - last[0];
                let dy = point[1] - last[1];
                (dx * dx + dy * dy).sqrt() > 0.5
            })
            .unwrap_or(true);
        if should_push_last {
            out.push(last);
        }
    }

    out
}

fn make_object_for_kind(
    kind: object_catalog::ObjectCatalogKind,
    x_raw: i16,
    y_raw: i16,
    z_raw: u8,
) -> ObjectInfo {
    let mut object = kind.create_default();
    let position = object.position_mut();
    position.x = x_raw;
    position.y = y_raw;
    position.z = z_raw;
    object
}

fn object_xy_units(object: &ObjectInfo) -> (f64, f64) {
    let pos = object.position();
    (pos.x as f64, pos.y as f64)
}

fn plot_to_raw_units(plot_value: f64) -> i16 {
    plot_value.round().clamp(i16::MIN as f64, i16::MAX as f64) as i16
}

fn normalize_lyt_path(mut path: PathBuf) -> PathBuf {
    if path.extension().is_none() {
        let _ = path.set_extension("lyt");
    }
    path
}

/// Reads an image from disk and uploads it to the GPU via egui.
fn load_image_to_texture(ctx: &egui::Context, path: &Path) -> Result<LoadedBackground, String> {
    let image_data = std::fs::read(path)
        .map_err(|e| format!("could not read '{}': {e}", path.display()))?;

    // Load as a DynamicImage first so we can check its size before converting to RGBA
    let mut dyn_image = image::load_from_memory(&image_data)
        .map_err(|e| format!("could not decode '{}': {e}", path.display()))?;

    let source_size_px = [dyn_image.width(), dyn_image.height()];

    let max_size = 2048;
    if dyn_image.width() > max_size || dyn_image.height() > max_size {
        // Resize keeping aspect ratio, using a fast filter
        dyn_image = dyn_image.resize(max_size, max_size, image::imageops::FilterType::Triangle);
    }

    let image = dyn_image.into_rgba8();

    let size = [image.width() as usize, image.height() as usize];
    let pixels = image.as_flat_samples();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

    Ok(LoadedBackground {
        texture: ctx.load_texture("track_map_bg", color_image, egui::TextureOptions::LINEAR),
        size_units: [
            source_size_px[0] as f32 * RAW_UNITS_PER_METRE,
            source_size_px[1] as f32 * RAW_UNITS_PER_METRE,
        ],
        source_size_px,
    })
}

fn main() -> eframe::Result<()> {
    // Set up standard window options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_title("LFS InSim Track Editor"),
        ..Default::default()
    };

    eframe::run_native(
        "LFS Track Editor",
        options,
        Box::new(|cc| Ok(Box::new(TrackEditor::new(cc)))),
    )
}
