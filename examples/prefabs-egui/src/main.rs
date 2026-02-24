//! Prefabs egui
#[allow(missing_docs, unused_results)]
use eframe::egui;
use egui_plot::{Plot, PlotImage, PlotPoint, PlotPoints, Points};
use insim_core::object::{ObjectCoordinate, ObjectInfo, Raw};
use insim_lyt::Lyt;
use std::path::{Path, PathBuf};

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

    fn insert_object(&mut self, object: ObjectInfo) {
        self.objects.push(object);
        self.object_ids.push(self.next_object_id);
        self.next_object_id += 1;
    }

    fn delete_selected_objects(&mut self) {
        use std::collections::HashSet;

        let selected: HashSet<u64> = self.tools.select.selected_object_ids.drain(..).collect();
        if selected.is_empty() {
            return;
        }

        let mut kept_objects = Vec::with_capacity(self.objects.len());
        let mut kept_ids = Vec::with_capacity(self.object_ids.len());

        for (object_id, object) in self.object_ids.drain(..).zip(self.objects.drain(..)) {
            if !selected.contains(&object_id) {
                kept_ids.push(object_id);
                kept_objects.push(object);
            }
        }

        self.object_ids = kept_ids;
        self.objects = kept_objects;
    }

    fn replace_objects(&mut self, objects: Vec<ObjectInfo>) {
        let len = objects.len() as u64;
        self.objects = objects;
        self.object_ids = (0..len).collect();
        self.next_object_id = len;
        self.tools.select.selected_object_ids.clear();
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
                        ui.heading("Selection Tool");
                        ui.label("Click an object on the map to edit its properties.");
                        ui.label(format!(
                            "Selected objects: {}",
                            self.tools.select.selected_object_ids.len()
                        ));
                    },
                    tools::ToolKind::Place => {
                        let place = &mut self.tools.place;
                        ui.heading("Object Library");
                        ui.horizontal(|ui| {
                            ui.label("🔍");
                            ui.text_edit_singleline(&mut place.search_query);
                        });
                        ui.separator();

                        egui::ScrollArea::vertical().show(ui, |ui| {
                            // Dummy list of objects to demonstrate filtering
                            let dummy_objects = [
                                (16_u8, "Painted Letters"),
                                (17_u8, "Painted Arrows"),
                                (20_u8, "Cone"),
                                (49_u8, "Tyre Stack 2"),
                                (104_u8, "Barrier Long"),
                                (172_u8, "Concrete Slab"),
                            ];

                            let query = place.search_query.to_lowercase();
                            for (id, name) in dummy_objects {
                                if query.is_empty() || name.to_lowercase().contains(&query) {
                                    ui.selectable_value(&mut place.selected_object_type, id, name);
                                }
                            }
                        });
                    },
                    tools::ToolKind::SplinePath => {
                        ui.heading("Catmull-Rom Spline");
                        ui.label("Click the map to place control nodes.");
                        ui.separator();
                        if ui.button("Clear Nodes").clicked() {
                            //self.spline_nodes.clear();
                        }
                        // ui.label(format!("Nodes placed: {}", self.spline_nodes.len()));
                        // We will add the "Generate Objects" button here next!
                    },
                    tools::ToolKind::RampGen => {
                        ui.heading("Ramp Generator");
                        ui.label("Select Start/End to build a ramp.");
                    },
                }
            });

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
            });

            // 3. Handle Interaction using the variable we captured
            if plot_response.response.clicked() {
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
                            let object = make_object_raw(
                                self.tools.place.selected_object_type,
                                click_x_raw,
                                click_y_raw,
                                DEFAULT_PLACE_Z_UNITS,
                            );
                            self.insert_object(object);
                        },
                        tools::ToolKind::SplinePath | tools::ToolKind::RampGen => {},
                    }
                }
            }
        });
    }
}

// --- 3. UTILITY FUNCTIONS ---

fn make_object_raw(type_id: u8, x_raw: i16, y_raw: i16, z_raw: u8) -> ObjectInfo {
    ObjectInfo::Unknown(Raw {
        index: type_id,
        xyz: ObjectCoordinate::new(x_raw, y_raw, z_raw),
        flags: 0,
        heading: 0,
    })
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
