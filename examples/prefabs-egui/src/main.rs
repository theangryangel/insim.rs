//! Prefabs egui
#[allow(missing_docs, unused_results)]
use eframe::egui;
use egui_plot::{Arrows, Plot, PlotImage, PlotPoint, PlotPoints, Points, Text};
use insim_core::{heading::Heading, object::ObjectInfo};
use insim_lyt::Lyt;
use std::path::{Path, PathBuf};

#[allow(missing_docs)]
/// Inspector UI helpers.
mod inspector;
/// Placeable object catalog.
mod object_catalog;
#[allow(missing_docs)]
/// Object library picker widget.
mod object_library_widget;
/// Ramp generation utilities.
mod ramp_gen;
/// Spline distribution utilities.
mod spline_distrib;
pub mod tools;

const CLICK_RADIUS_UNITS: i64 = 160;
const DEFAULT_PLACE_Z_UNITS: u8 = 0;
const HEADING_ARROW_LENGTH_UNITS: f64 = 24.0;
const HEADING_LABEL_OFFSET_UNITS: f64 = 8.0;
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
}

impl TrackEditor {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let _ = cc;

        Self {
            objects: Vec::new(),
            object_ids: Vec::new(),
            next_object_id: 0,
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
        self.tools
            .ramp_gen
            .generated_object_ids
            .retain(|id| !targets.contains(id));
        removed
    }

    fn delete_selected_objects(&mut self) {
        let selected_ids = std::mem::take(&mut self.tools.select.selected_object_ids);
        let _ = self.delete_objects_by_ids(&selected_ids);
    }

    fn apply_tool_event(&mut self, event: tools::ToolEvent) {
        match event {
            tools::ToolEvent::DeleteSelectedObjects => {
                self.delete_selected_objects();
            },
            tools::ToolEvent::ApplySplineObjects => {
                self.apply_spline_objects();
            },
            tools::ToolEvent::ClearSplineGeneratedObjects => {
                let removed = self.clear_spline_generated_objects();
                self.status_line = Some(format!("Spline generated objects cleared: {}", removed));
            },
            tools::ToolEvent::ApplyRampObjects => {
                self.apply_ramp_objects();
            },
            tools::ToolEvent::ClearRampGeneratedObjects => {
                let removed = self.clear_ramp_generated_objects();
                self.status_line = Some(format!("Ramp generated objects cleared: {}", removed));
            },
            tools::ToolEvent::SetSelectionFromClick {
                object_id,
                additive,
            } => {
                if let Some(object_id) = object_id {
                    let selected = &mut self.tools.select.selected_object_ids;
                    if additive {
                        if let Some(existing) =
                            selected.iter().position(|selected| *selected == object_id)
                        {
                            let _ = selected.remove(existing);
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
            tools::ToolEvent::PlaceObject(request) => {
                let object = request.into_object();
                let object_id = self.insert_object(object);
                self.tools.select.selected_object_ids.clear();
                self.tools.select.selected_object_ids.push(object_id);
                self.tools.activate(tools::ToolKind::Select);
            },
            tools::ToolEvent::AddSplineControlPoint(point) => {
                self.tools.spline_path.control_points.push(point);
            },
            tools::ToolEvent::UpsertRampControlPoint {
                x_raw,
                y_raw,
                click_radius_sq,
                default_z_raw,
            } => {
                let mut closest_idx = None;
                let mut closest_dist_sq = i64::MAX;
                for (idx, node) in self.tools.ramp_gen.control_points.iter().enumerate() {
                    let dx = i64::from(node.x) - i64::from(x_raw);
                    let dy = i64::from(node.y) - i64::from(y_raw);
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq < click_radius_sq && dist_sq < closest_dist_sq {
                        closest_dist_sq = dist_sq;
                        closest_idx = Some(idx);
                    }
                }

                if let Some(idx) = closest_idx {
                    self.tools.ramp_gen.selected_node = Some(idx);
                } else {
                    let inherited_z = self
                        .tools
                        .ramp_gen
                        .control_points
                        .last()
                        .map(|node| node.z)
                        .unwrap_or(default_z_raw);
                    self.tools.ramp_gen.control_points.push(tools::RampNode {
                        x: x_raw,
                        y: y_raw,
                        z: inherited_z,
                    });
                    self.tools.ramp_gen.selected_node =
                        Some(self.tools.ramp_gen.control_points.len() - 1);
                }
            },
        }
    }

    fn replace_objects(&mut self, objects: Vec<ObjectInfo>) {
        let len = objects.len() as u64;
        self.objects = objects;
        self.object_ids = (0..len).collect();
        self.next_object_id = len;
        self.tools.select.selected_object_ids.clear();
        self.tools.spline_path.generated_object_ids.clear();
        self.tools.ramp_gen.generated_object_ids.clear();
    }

    fn clear_spline_generated_objects(&mut self) -> usize {
        let ids = std::mem::take(&mut self.tools.spline_path.generated_object_ids);
        self.delete_objects_by_ids(&ids)
    }

    fn clear_ramp_generated_objects(&mut self) -> usize {
        let ids = std::mem::take(&mut self.tools.ramp_gen.generated_object_ids);
        self.delete_objects_by_ids(&ids)
    }

    fn apply_spline_objects(&mut self) {
        let control_points = self.tools.spline_path.control_points.clone();
        let spacing_units = self.tools.spline_path.spacing_units;
        let template = self.tools.spline_path.object_template.clone();
        let generated = match spline_distrib::build(&control_points, &template, spacing_units, None)
        {
            Ok(objects) => objects,
            Err(err) => {
                self.status_line = Some(format!("Spline apply failed: {err}"));
                return;
            },
        };

        if generated.is_empty() {
            self.status_line = Some("Spline apply produced no objects.".to_owned());
            return;
        }

        let old_generated = self.tools.spline_path.generated_object_ids.clone();
        let removed = self.delete_objects_by_ids(&old_generated);

        let mut new_generated_ids = Vec::with_capacity(generated.len());
        for object in generated {
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

    fn apply_ramp_objects(&mut self) {
        let control_points = self.tools.ramp_gen.control_points.clone();
        if control_points.len() < 2 {
            self.status_line = Some("Ramp apply needs at least 2 control points.".to_owned());
            return;
        }

        let template = self.tools.ramp_gen.object_template.clone();
        let steps = self.tools.ramp_gen.steps_per_segment;

        let generated = match ramp_gen::build(&control_points, &template, steps) {
            Ok(objects) => objects,
            Err(err) => {
                self.status_line = Some(format!("Ramp apply failed: {err}"));
                return;
            },
        };

        if generated.is_empty() {
            self.status_line = Some("Ramp apply produced no objects.".to_owned());
            return;
        }

        let old_generated = self.tools.ramp_gen.generated_object_ids.clone();
        let removed = self.delete_objects_by_ids(&old_generated);

        let mut generated_ids = Vec::with_capacity(generated.len());
        for object in generated {
            generated_ids.push(self.insert_object(object));
        }

        self.tools.select.selected_object_ids = generated_ids.clone();
        self.tools.ramp_gen.generated_object_ids = generated_ids;
        self.status_line = Some(format!(
            "Ramp applied: {} object(s) generated, {} cleared.",
            self.tools.ramp_gen.generated_object_ids.len(),
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
                self.status_line = Some(format!(
                    "Loaded LYT ({} objects): {}",
                    count,
                    path.display()
                ));
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

        let mut tool_events = self.tools.gather_shortcut_events(ctx);
        self.tools.clear_transient_state();
        self.tools.show_tool_palette_panel(ctx);
        tool_events.extend(
            self.tools
                .show_context_panel(ctx, &self.object_ids, &mut self.objects),
        );

        for event in tool_events {
            self.apply_tool_event(event);
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
            let spline_curve = spline_distrib::preview_curve_points(&spline_control_points);
            let spline_preview_samples = spline_distrib::sample_spaced_raw(
                &spline_control_points,
                self.tools.spline_path.spacing_units,
                None,
            )
            .unwrap_or_default();

            let ramp_control_points = self.tools.ramp_gen.control_points.clone();
            let ramp_selected_node = self.tools.ramp_gen.selected_node;
            let ramp_xy_points: Vec<[i16; 2]> = ramp_control_points
                .iter()
                .map(|node| [node.x, node.y])
                .collect();
            let ramp_curve = spline_distrib::preview_curve_points(&ramp_xy_points);

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
                let mut selected_points = Vec::new();
                let mut selected_heading_origins = Vec::new();
                let mut selected_heading_tips = Vec::new();
                let mut selected_heading_labels = Vec::new();

                for (object_id, object) in self.object_ids.iter().copied().zip(self.objects.iter())
                {
                    let (x, y) = object_xy_units(object);
                    if selected_lookup.contains(&object_id) {
                        selected_points.push([x, y]);
                        if let Some(heading) = object.heading() {
                            let heading_degrees = heading.normalize().to_degrees();
                            let [dir_x, dir_y] = heading_to_plot_direction(heading);
                            let tip = [
                                x + dir_x * HEADING_ARROW_LENGTH_UNITS,
                                y + dir_y * HEADING_ARROW_LENGTH_UNITS,
                            ];
                            selected_heading_origins.push([x, y]);
                            selected_heading_tips.push(tip);
                            selected_heading_labels.push((
                                object_id,
                                [
                                    tip[0] + dir_x * HEADING_LABEL_OFFSET_UNITS,
                                    tip[1] + dir_y * HEADING_LABEL_OFFSET_UNITS,
                                ],
                                format!("{heading_degrees:.1}°"),
                            ));
                        }
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
                    Points::new("Selected", PlotPoints::new(selected_points))
                        .radius(6.0)
                        .color(egui::Color32::YELLOW)
                        .name("Selected"),
                );

                if !selected_heading_origins.is_empty() {
                    plot_ui.arrows(
                        Arrows::new(
                            "Selected Heading",
                            PlotPoints::new(selected_heading_origins),
                            PlotPoints::new(selected_heading_tips),
                        )
                        .tip_length(10.0)
                        .color(egui::Color32::from_rgb(255, 220, 120)),
                    );

                    for (object_id, label_position, label_text) in selected_heading_labels {
                        plot_ui.text(
                            Text::new(
                                format!("Heading {object_id}"),
                                PlotPoint::new(label_position[0], label_position[1]),
                                label_text,
                            )
                            .anchor(egui::Align2::CENTER_CENTER)
                            .color(egui::Color32::from_rgb(255, 240, 180)),
                        );
                    }
                }

                if let (Some(start), Some(current)) =
                    (self.tools.select.drag_start, self.tools.select.drag_current)
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

                if !ramp_curve.is_empty() {
                    plot_ui.line(
                        egui_plot::Line::new("Ramp Curve", PlotPoints::new(ramp_curve.clone()))
                            .color(egui::Color32::from_rgb(120, 220, 120)),
                    );
                }

                if !ramp_control_points.is_empty() {
                    let ramp_points: Vec<[f64; 2]> = ramp_control_points
                        .iter()
                        .map(|node| [f64::from(node.x), f64::from(node.y)])
                        .collect();
                    plot_ui.points(
                        Points::new("Ramp Control", PlotPoints::new(ramp_points.clone()))
                            .radius(5.0)
                            .color(egui::Color32::from_rgb(160, 255, 140)),
                    );

                    if let Some(selected_idx) = ramp_selected_node
                        && selected_idx < ramp_points.len()
                    {
                        plot_ui.points(
                            Points::new(
                                "Ramp Selected Control",
                                PlotPoints::new(vec![ramp_points[selected_idx]]),
                            )
                            .radius(7.0)
                            .color(egui::Color32::YELLOW),
                        );
                    }
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
                        self.tools.select.begin_drag(drag_start, true);
                    }
                }

                if self.tools.select.drag_start.is_some() && plot_response.response.dragged() {
                    if let Some(pointer_pos) = pointer_coord {
                        self.tools.select.update_drag([
                            plot_to_raw_units(pointer_pos.x),
                            plot_to_raw_units(pointer_pos.y),
                        ]);
                    }
                }

                if self.tools.select.drag_start.is_some() && plot_response.response.drag_stopped() {
                    self.tools
                        .select
                        .commit_drag_selection(&self.object_ids, &self.objects);
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

                    let additive_selection =
                        ctx.input(|i| i.modifiers.command || i.modifiers.shift);
                    let click_events = self.tools.map_click_events(
                        click_x_raw,
                        click_y_raw,
                        closest_object_id,
                        additive_selection,
                        DEFAULT_PLACE_Z_UNITS,
                        click_radius_sq,
                    );
                    for event in click_events {
                        self.apply_tool_event(event);
                    }
                }
            }
        });
    }
}

// --- 3. UTILITY FUNCTIONS ---

fn object_xy_units(object: &ObjectInfo) -> (f64, f64) {
    let pos = object.position();
    (pos.x as f64, pos.y as f64)
}

fn heading_to_plot_direction(heading: Heading) -> [f64; 2] {
    let radians = heading.to_radians();
    [-radians.sin(), radians.cos()]
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
    let image_data =
        std::fs::read(path).map_err(|e| format!("could not read '{}': {e}", path.display()))?;

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
