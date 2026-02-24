use eframe::egui;
use insim_core::{
    heading::Heading,
    object::{concrete, control, insim, marker, marshal, painted, sign_metal, sign_speed, ObjectInfo},
};

/// Shared object editor options.
#[derive(Debug, Clone, Copy)]
pub struct ObjectEditorOptions {
    /// Show object type title.
    pub show_title: bool,
    /// Show position editor.
    pub show_position: bool,
    /// Show debug dump.
    pub show_debug: bool,
}

impl Default for ObjectEditorOptions {
    fn default() -> Self {
        Self {
            show_title: true,
            show_position: true,
            show_debug: true,
        }
    }
}

impl ObjectEditorOptions {
    /// Options for template editing.
    pub fn template() -> Self {
        Self {
            show_title: true,
            show_position: false,
            show_debug: false,
        }
    }
}

/// Reusable position editor widget.
pub struct PositionEditorWidget<'a> {
    position: &'a mut insim_core::object::ObjectCoordinate,
}

impl<'a> PositionEditorWidget<'a> {
    /// Creates a new position editor.
    pub fn new(position: &'a mut insim_core::object::ObjectCoordinate) -> Self {
        Self { position }
    }
}

impl egui::Widget for PositionEditorWidget<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            let _ = ui.label("X");
            let _ = ui.add(egui::DragValue::new(&mut self.position.x).speed(1.0));
            let _ = ui.label("Y");
            let _ = ui.add(egui::DragValue::new(&mut self.position.y).speed(1.0));
            let _ = ui.label("Z");
            let _ = ui.add(egui::DragValue::new(&mut self.position.z).speed(1.0));
        })
        .response
    }
}

/// Reusable object editor widget.
pub struct ObjectEditorWidget<'a> {
    object: &'a mut ObjectInfo,
    options: ObjectEditorOptions,
}

impl<'a> ObjectEditorWidget<'a> {
    /// Creates a new object editor.
    pub fn new(object: &'a mut ObjectInfo) -> Self {
        Self {
            object,
            options: ObjectEditorOptions::default(),
        }
    }

    /// Overrides display options.
    pub fn options(mut self, options: ObjectEditorOptions) -> Self {
        self.options = options;
        self
    }
}

impl egui::Widget for ObjectEditorWidget<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            if self.options.show_title {
                let _ = ui.label(format!("Type: {}", object_info_kind_name(self.object)));
            }

            if self.options.show_position {
                let _ = ui.separator();
                let _ = ui.label("Position (raw units)");
                let _ = ui.add(PositionEditorWidget::new(self.object.position_mut()));
            }

            if let Some(heading) = self.object.heading_mut() {
                let mut heading_degrees = heading.to_degrees();
                let _ = ui.horizontal(|ui| {
                    let _ = ui.label("Heading (degrees)");
                    let response = ui.add(egui::DragValue::new(&mut heading_degrees).speed(1.0));
                    if response.changed() {
                        *heading = Heading::from_degrees(heading_degrees);
                    }
                });
            } else {
                let _ = ui.label("Heading: n/a");
            }

            if let Some(floating) = self.object.floating() {
                let _ = ui.label(format!("Floating: {}", floating));
            }

            edit_specific_object_fields(ui, self.object);

            if self.options.show_debug {
                let _ = ui.collapsing("Debug", |ui| {
                    let _ = ui.monospace(format!("{:#?}", self.object));
                });
            }
        })
        .response
    }
}

/// Renders selection details and basic editable fields.
pub fn show_selection_inspector(
    ui: &mut egui::Ui,
    object_ids: &[u64],
    objects: &mut [ObjectInfo],
    selected_object_ids: &[u64],
) {
    let _ = ui.heading("Selection Tool");
    let _ = ui.label("Click an object on the map to inspect it.");
    let _ = ui.label(format!("Selected objects: {}", selected_object_ids.len()));

    if selected_object_ids.is_empty() {
        return;
    }

    if selected_object_ids.len() > 1 {
        let _ = ui.separator();
        let _ = ui.label("Multi-select active. Single-select for full inspector details.");
        return;
    }

    let object_id = selected_object_ids[0];
    let Some(object_index) = object_ids
        .iter()
        .position(|candidate_id| *candidate_id == object_id)
    else {
        let _ = ui.separator();
        let _ = ui.colored_label(
            egui::Color32::LIGHT_RED,
            format!("Selected object id {} is not present.", object_id),
        );
        return;
    };

    let object = &mut objects[object_index];

    let _ = ui.separator();
    let _ = ui.label(format!("Object ID: {}", object_id));
    let _ = ui.add(ObjectEditorWidget::new(object));
}

pub fn object_info_kind_name(object: &ObjectInfo) -> &'static str {
    match object {
        ObjectInfo::Control(_) => "Control",
        ObjectInfo::Marshal(_) => "Marshal",
        ObjectInfo::InsimCheckpoint(_) => "InSim Checkpoint",
        ObjectInfo::InsimCircle(_) => "InSim Circle",
        ObjectInfo::RestrictedArea(_) => "Restricted Area",
        ObjectInfo::RouteChecker(_) => "Route Checker",
        ObjectInfo::ChalkLine(_)
        | ObjectInfo::ChalkLine2(_)
        | ObjectInfo::ChalkAhead(_)
        | ObjectInfo::ChalkAhead2(_)
        | ObjectInfo::ChalkLeft(_)
        | ObjectInfo::ChalkLeft2(_)
        | ObjectInfo::ChalkLeft3(_)
        | ObjectInfo::ChalkRight(_)
        | ObjectInfo::ChalkRight2(_)
        | ObjectInfo::ChalkRight3(_) => "Chalk",
        ObjectInfo::PaintLetters(_) => "Painted Letters",
        ObjectInfo::PaintArrows(_) => "Painted Arrows",
        ObjectInfo::Cone1(_)
        | ObjectInfo::Cone2(_)
        | ObjectInfo::ConeTall1(_)
        | ObjectInfo::ConeTall2(_)
        | ObjectInfo::ConePointer(_) => "Cone",
        ObjectInfo::TyreSingle(_)
        | ObjectInfo::TyreStack2(_)
        | ObjectInfo::TyreStack3(_)
        | ObjectInfo::TyreStack4(_)
        | ObjectInfo::TyreSingleBig(_)
        | ObjectInfo::TyreStack2Big(_)
        | ObjectInfo::TyreStack3Big(_)
        | ObjectInfo::TyreStack4Big(_) => "Tyres",
        ObjectInfo::MarkerCorner(_) => "Marker Corner",
        ObjectInfo::MarkerDistance(_) => "Marker Distance",
        ObjectInfo::LetterboardWY(_) => "Letterboard WY",
        ObjectInfo::LetterboardRB(_) => "Letterboard RB",
        ObjectInfo::Armco1(_) | ObjectInfo::Armco3(_) | ObjectInfo::Armco5(_) => "Armco",
        ObjectInfo::BarrierLong(_) | ObjectInfo::BarrierRed(_) | ObjectInfo::BarrierWhite(_) => {
            "Barrier"
        },
        ObjectInfo::Banner(_) => "Banner",
        ObjectInfo::Ramp1(_) | ObjectInfo::Ramp2(_) => "Ramp",
        ObjectInfo::VehicleSUV(_) => "Vehicle SUV",
        ObjectInfo::VehicleVan(_) => "Vehicle Van",
        ObjectInfo::VehicleTruck(_) => "Vehicle Truck",
        ObjectInfo::VehicleAmbulance(_) => "Vehicle Ambulance",
        ObjectInfo::SpeedHump10M(_)
        | ObjectInfo::SpeedHump6M(_)
        | ObjectInfo::SpeedHump2M(_)
        | ObjectInfo::SpeedHump1M(_) => "Speed Hump",
        ObjectInfo::Kerb(_) => "Kerb",
        ObjectInfo::Post(_) => "Post",
        ObjectInfo::Marquee(_) => "Marquee",
        ObjectInfo::Bale(_) => "Bale",
        ObjectInfo::Bin1(_) => "Bin1",
        ObjectInfo::Bin2(_) => "Bin2",
        ObjectInfo::Railing1(_) | ObjectInfo::Railing2(_) => "Railing",
        ObjectInfo::StartLights1(_) | ObjectInfo::StartLights2(_) | ObjectInfo::StartLights3(_) => {
            "Start Lights"
        },
        ObjectInfo::SignMetal(_) => "Sign Metal",
        ObjectInfo::ChevronLeft(_) | ObjectInfo::ChevronRight(_) => "Chevron",
        ObjectInfo::SignSpeed(_) => "Sign Speed",
        ObjectInfo::ConcreteSlab(_) => "Concrete Slab",
        ObjectInfo::ConcreteRamp(_) => "Concrete Ramp",
        ObjectInfo::ConcreteWall(_) => "Concrete Wall",
        ObjectInfo::ConcretePillar(_) => "Concrete Pillar",
        ObjectInfo::ConcreteSlabWall(_) => "Concrete Slab Wall",
        ObjectInfo::ConcreteRampWall(_) => "Concrete Ramp Wall",
        ObjectInfo::ConcreteShortSlabWall(_) => "Concrete Short Slab Wall",
        ObjectInfo::ConcreteWedge(_) => "Concrete Wedge",
        ObjectInfo::StartPosition(_) => "Start Position",
        ObjectInfo::PitStartPoint(_) => "Pit Start Point",
        ObjectInfo::PitStopBox(_) => "Pit Stop Box",
        ObjectInfo::Unknown(_) => "Unknown",
        _ => "Unhandled",
    }
}

fn section_title(ui: &mut egui::Ui, title: &str) {
    let _ = ui.separator();
    let _ = ui.label(title);
}

fn floating_checkbox(ui: &mut egui::Ui, value: &mut bool) {
    let _ = ui.checkbox(value, "Floating");
}

fn edit_specific_object_fields(ui: &mut egui::Ui, object: &mut ObjectInfo) {
    match object {
        ObjectInfo::ConcreteSlab(payload) => {
            section_title(ui, "Concrete Slab");
            edit_concrete_width(ui, "Width", &mut payload.width);
            edit_concrete_width(ui, "Length", &mut payload.length);
            edit_concrete_pitch(ui, "Pitch", &mut payload.pitch);
        },
        ObjectInfo::ConcreteRamp(payload) => {
            section_title(ui, "Concrete Ramp");
            edit_concrete_width(ui, "Width", &mut payload.width);
            edit_concrete_width(ui, "Length", &mut payload.length);
            edit_concrete_height(ui, "Height", &mut payload.height);
        },
        ObjectInfo::ConcreteWall(payload) => {
            section_title(ui, "Concrete Wall");
            edit_concrete_colour(ui, "Colour", &mut payload.colour);
            edit_concrete_width(ui, "Length", &mut payload.length);
            edit_concrete_height(ui, "Height", &mut payload.height);
        },
        ObjectInfo::ConcretePillar(payload) => {
            section_title(ui, "Concrete Pillar");
            edit_size(ui, "Size X", &mut payload.x);
            edit_size(ui, "Size Y", &mut payload.y);
            edit_concrete_height(ui, "Height", &mut payload.height);
        },
        ObjectInfo::ConcreteSlabWall(payload) => {
            section_title(ui, "Concrete Slab Wall");
            edit_concrete_colour(ui, "Colour", &mut payload.colour);
            edit_concrete_width(ui, "Length", &mut payload.length);
            edit_concrete_pitch(ui, "Pitch", &mut payload.pitch);
        },
        ObjectInfo::ConcreteRampWall(payload) => {
            section_title(ui, "Concrete Ramp Wall");
            edit_concrete_colour(ui, "Colour", &mut payload.colour);
            edit_concrete_width(ui, "Length", &mut payload.length);
            edit_concrete_height(ui, "Height", &mut payload.height);
        },
        ObjectInfo::ConcreteShortSlabWall(payload) => {
            section_title(ui, "Concrete Short Slab Wall");
            edit_concrete_colour(ui, "Colour", &mut payload.colour);
            edit_size(ui, "Size Y", &mut payload.y);
            edit_concrete_pitch(ui, "Pitch", &mut payload.pitch);
        },
        ObjectInfo::ConcreteWedge(payload) => {
            section_title(ui, "Concrete Wedge");
            edit_concrete_colour(ui, "Colour", &mut payload.colour);
            edit_concrete_width(ui, "Length", &mut payload.length);
            edit_concrete_angle(ui, "Angle", &mut payload.angle);
        },
        ObjectInfo::Ramp1(payload) | ObjectInfo::Ramp2(payload) => {
            section_title(ui, "Ramp");
            edit_u8_range(ui, "Colour", &mut payload.colour, 0, 7);
            edit_u8_range(ui, "Mapping", &mut payload.mapping, 0, 15);
            floating_checkbox(ui, &mut payload.floating);
        },
        ObjectInfo::BarrierLong(payload)
        | ObjectInfo::BarrierRed(payload)
        | ObjectInfo::BarrierWhite(payload) => {
            section_title(ui, "Barrier");
            edit_u8_range(ui, "Colour", &mut payload.colour, 0, 7);
            edit_u8_range(ui, "Mapping", &mut payload.mapping, 0, 15);
            floating_checkbox(ui, &mut payload.floating);
        },
        ObjectInfo::Cone1(payload)
        | ObjectInfo::Cone2(payload)
        | ObjectInfo::ConeTall1(payload)
        | ObjectInfo::ConeTall2(payload)
        | ObjectInfo::ConePointer(payload) => {
            section_title(ui, "Cone");
            edit_cone_colour(ui, "Colour", &mut payload.colour);
            floating_checkbox(ui, &mut payload.floating);
        },
        ObjectInfo::TyreSingle(payload)
        | ObjectInfo::TyreStack2(payload)
        | ObjectInfo::TyreStack3(payload)
        | ObjectInfo::TyreStack4(payload)
        | ObjectInfo::TyreSingleBig(payload)
        | ObjectInfo::TyreStack2Big(payload)
        | ObjectInfo::TyreStack3Big(payload)
        | ObjectInfo::TyreStack4Big(payload) => {
            section_title(ui, "Tyres");
            edit_tyre_colour(ui, "Colour", &mut payload.colour);
            floating_checkbox(ui, &mut payload.floating);
        },
        ObjectInfo::SignMetal(payload) => {
            section_title(ui, "Metal Sign");
            edit_metal_sign_kind(ui, "Kind", &mut payload.kind);
            edit_u8_range(ui, "Colour", &mut payload.colour, 0, 7);
            floating_checkbox(ui, &mut payload.floating);
        },
        ObjectInfo::SignSpeed(payload) => {
            section_title(ui, "Speed Sign");
            edit_speed_sign_mapping(ui, "Sign", &mut payload.mapping);
            edit_u8_range(ui, "Colour", &mut payload.colour, 0, 7);
            floating_checkbox(ui, &mut payload.floating);
        },
        ObjectInfo::StartPosition(payload) => {
            section_title(ui, "Start Position");
            edit_u8_range(ui, "Index", &mut payload.index, 0, 47);
            floating_checkbox(ui, &mut payload.floating);
        },
        ObjectInfo::PitStartPoint(payload) => {
            section_title(ui, "Pit Start Point");
            edit_u8_range(ui, "Index", &mut payload.index, 0, 47);
            floating_checkbox(ui, &mut payload.floating);
        },
        ObjectInfo::PitStopBox(payload) => {
            section_title(ui, "Pit Stop Box");
            edit_u8_range(ui, "Colour", &mut payload.colour, 0, 7);
            edit_u8_range(ui, "Mapping", &mut payload.mapping, 0, 15);
            floating_checkbox(ui, &mut payload.floating);
        },
        ObjectInfo::StartLights1(payload)
        | ObjectInfo::StartLights2(payload)
        | ObjectInfo::StartLights3(payload) => {
            section_title(ui, "Start Lights");
            edit_u8_range(ui, "Identifier", &mut payload.identifier, 0, 63);
            floating_checkbox(ui, &mut payload.floating);
        },
        ObjectInfo::MarkerCorner(payload) => {
            section_title(ui, "Corner Marker");
            edit_marker_corner_kind(ui, "Kind", &mut payload.kind);
            floating_checkbox(ui, &mut payload.floating);
        },
        ObjectInfo::MarkerDistance(payload) => {
            section_title(ui, "Distance Marker");
            edit_marker_distance_kind(ui, "Distance", &mut payload.kind);
            floating_checkbox(ui, &mut payload.floating);
        },
        ObjectInfo::PaintLetters(payload) => {
            section_title(ui, "Painted Letters");
            edit_paint_colour(ui, "Colour", &mut payload.colour);
            edit_painted_character(ui, "Character", &mut payload.character);
            floating_checkbox(ui, &mut payload.floating);
        },
        ObjectInfo::PaintArrows(payload) => {
            section_title(ui, "Painted Arrows");
            edit_paint_colour(ui, "Colour", &mut payload.colour);
            edit_paint_arrow(ui, "Arrow", &mut payload.arrow);
            floating_checkbox(ui, &mut payload.floating);
        },
        ObjectInfo::LetterboardRB(payload) => {
            section_title(ui, "Letterboard RB");
            edit_letterboard_rb_colour(ui, "Colour", &mut payload.colour);
            edit_letterboard_character(ui, "Character", &mut payload.character);
            floating_checkbox(ui, &mut payload.floating);
        },
        ObjectInfo::LetterboardWY(payload) => {
            section_title(ui, "Letterboard WY");
            edit_letterboard_wy_colour(ui, "Colour", &mut payload.colour);
            edit_letterboard_character(ui, "Character", &mut payload.character);
            floating_checkbox(ui, &mut payload.floating);
        },
        ObjectInfo::InsimCheckpoint(payload) => {
            section_title(ui, "InSim Checkpoint");
            edit_insim_checkpoint_kind(ui, "Kind", &mut payload.kind);
            floating_checkbox(ui, &mut payload.floating);
        },
        ObjectInfo::InsimCircle(payload) => {
            section_title(ui, "InSim Circle");
            edit_u8_range(ui, "Index", &mut payload.index, 0, u8::MAX);
            floating_checkbox(ui, &mut payload.floating);
        },
        ObjectInfo::Control(payload) => {
            section_title(ui, "Control");
            edit_control_kind(ui, &mut payload.kind);
            floating_checkbox(ui, &mut payload.floating);
        },
        ObjectInfo::Marshal(payload) => {
            section_title(ui, "Marshal");
            edit_marshal_kind(ui, "Kind", &mut payload.kind);
            floating_checkbox(ui, &mut payload.floating);
        },
        ObjectInfo::RestrictedArea(payload) => {
            section_title(ui, "Restricted Area");
            edit_u8_range(ui, "Radius", &mut payload.radius, 0, 31);
            floating_checkbox(ui, &mut payload.floating);
        },
        ObjectInfo::RouteChecker(payload) => {
            section_title(ui, "Route Checker");
            edit_u8_range(ui, "Route", &mut payload.route, 0, u8::MAX);
            edit_u8_range(ui, "Radius", &mut payload.radius, 0, 31);
            floating_checkbox(ui, &mut payload.floating);
        },
        _ => {
            section_title(ui, "No object-specific fields for this type yet.");
        },
    }
}

fn edit_u8_range(ui: &mut egui::Ui, label: &str, value: &mut u8, min: u8, max: u8) {
    let mut raw = i32::from(*value);
    let response = ui.add(egui::Slider::new(&mut raw, i32::from(min)..=i32::from(max)).text(label));
    if response.changed() {
        *value = raw as u8;
    }
}

fn edit_concrete_width(ui: &mut egui::Ui, label: &str, value: &mut concrete::ConcreteWidthLength) {
    let _ = egui::ComboBox::from_label(label)
        .selected_text(concrete_width_name(*value))
        .show_ui(ui, |ui| {
            let _ = ui.selectable_value(value, concrete::ConcreteWidthLength::Two, "2m");
            let _ = ui.selectable_value(value, concrete::ConcreteWidthLength::Four, "4m");
            let _ = ui.selectable_value(value, concrete::ConcreteWidthLength::Eight, "8m");
            let _ = ui.selectable_value(value, concrete::ConcreteWidthLength::Sixteen, "16m");
        });
}

fn concrete_width_name(value: concrete::ConcreteWidthLength) -> &'static str {
    match value {
        concrete::ConcreteWidthLength::Two => "2m",
        concrete::ConcreteWidthLength::Four => "4m",
        concrete::ConcreteWidthLength::Eight => "8m",
        concrete::ConcreteWidthLength::Sixteen => "16m",
        _ => "Unknown",
    }
}

fn edit_concrete_colour(ui: &mut egui::Ui, label: &str, value: &mut concrete::ConcreteColour) {
    let _ = egui::ComboBox::from_label(label)
        .selected_text(concrete_colour_name(*value))
        .show_ui(ui, |ui| {
            let _ = ui.selectable_value(value, concrete::ConcreteColour::Grey, "Grey");
            let _ = ui.selectable_value(value, concrete::ConcreteColour::Red, "Red");
            let _ = ui.selectable_value(value, concrete::ConcreteColour::Blue, "Blue");
            let _ = ui.selectable_value(value, concrete::ConcreteColour::Yellow, "Yellow");
        });
}

fn concrete_colour_name(value: concrete::ConcreteColour) -> &'static str {
    match value {
        concrete::ConcreteColour::Grey => "Grey",
        concrete::ConcreteColour::Red => "Red",
        concrete::ConcreteColour::Blue => "Blue",
        concrete::ConcreteColour::Yellow => "Yellow",
        _ => "Unknown",
    }
}

fn edit_concrete_height(ui: &mut egui::Ui, label: &str, value: &mut concrete::ConcreteHeight) {
    let mut index = *value as u8;
    edit_u8_range(ui, label, &mut index, 0, 15);
    *value = concrete_height_from_index(index);
}

fn concrete_height_from_index(index: u8) -> concrete::ConcreteHeight {
    match index {
        0 => concrete::ConcreteHeight::M0_25,
        1 => concrete::ConcreteHeight::M0_50,
        2 => concrete::ConcreteHeight::M0_75,
        3 => concrete::ConcreteHeight::M1_00,
        4 => concrete::ConcreteHeight::M1_25,
        5 => concrete::ConcreteHeight::M1_50,
        6 => concrete::ConcreteHeight::M1_75,
        7 => concrete::ConcreteHeight::M2_00,
        8 => concrete::ConcreteHeight::M2_25,
        9 => concrete::ConcreteHeight::M2_50,
        10 => concrete::ConcreteHeight::M2_75,
        11 => concrete::ConcreteHeight::M3_00,
        12 => concrete::ConcreteHeight::M3_25,
        13 => concrete::ConcreteHeight::M3_50,
        14 => concrete::ConcreteHeight::M3_75,
        _ => concrete::ConcreteHeight::M4_00,
    }
}

fn edit_concrete_pitch(ui: &mut egui::Ui, label: &str, value: &mut concrete::ConcretePitch) {
    let mut index = *value as u8;
    edit_u8_range(ui, label, &mut index, 0, 15);
    *value = concrete_pitch_from_index(index);
}

fn concrete_pitch_from_index(index: u8) -> concrete::ConcretePitch {
    match index {
        0 => concrete::ConcretePitch::Deg0,
        1 => concrete::ConcretePitch::Deg6,
        2 => concrete::ConcretePitch::Deg12,
        3 => concrete::ConcretePitch::Deg18,
        4 => concrete::ConcretePitch::Deg24,
        5 => concrete::ConcretePitch::Deg30,
        6 => concrete::ConcretePitch::Deg36,
        7 => concrete::ConcretePitch::Deg42,
        8 => concrete::ConcretePitch::Deg48,
        9 => concrete::ConcretePitch::Deg54,
        10 => concrete::ConcretePitch::Deg60,
        11 => concrete::ConcretePitch::Deg66,
        12 => concrete::ConcretePitch::Deg72,
        13 => concrete::ConcretePitch::Deg78,
        14 => concrete::ConcretePitch::Deg84,
        _ => concrete::ConcretePitch::Deg90,
    }
}

fn edit_concrete_angle(ui: &mut egui::Ui, label: &str, value: &mut concrete::ConcreteAngle) {
    let mut index = *value as u8;
    edit_u8_range(ui, label, &mut index, 0, 15);
    *value = concrete_angle_from_index(index);
}

fn concrete_angle_from_index(index: u8) -> concrete::ConcreteAngle {
    match index {
        0 => concrete::ConcreteAngle::Deg5_625,
        1 => concrete::ConcreteAngle::Deg11_25,
        2 => concrete::ConcreteAngle::Deg16_875,
        3 => concrete::ConcreteAngle::Deg22_5,
        4 => concrete::ConcreteAngle::Deg28_125,
        5 => concrete::ConcreteAngle::Deg33_75,
        6 => concrete::ConcreteAngle::Deg39_375,
        7 => concrete::ConcreteAngle::Deg45,
        8 => concrete::ConcreteAngle::Deg50_625,
        9 => concrete::ConcreteAngle::Deg56_25,
        10 => concrete::ConcreteAngle::Deg61_875,
        11 => concrete::ConcreteAngle::Deg67_5,
        12 => concrete::ConcreteAngle::Deg73_125,
        13 => concrete::ConcreteAngle::Deg78_75,
        14 => concrete::ConcreteAngle::Deg84_375,
        _ => concrete::ConcreteAngle::Deg90,
    }
}

fn edit_size(ui: &mut egui::Ui, label: &str, value: &mut concrete::Size) {
    let _ = egui::ComboBox::from_label(label)
        .selected_text(size_name(*value))
        .show_ui(ui, |ui| {
            let _ = ui.selectable_value(value, concrete::Size::Quarter, "0.25");
            let _ = ui.selectable_value(value, concrete::Size::Half, "0.50");
            let _ = ui.selectable_value(value, concrete::Size::ThreeQuarter, "0.75");
            let _ = ui.selectable_value(value, concrete::Size::Full, "1.00");
        });
}

fn size_name(value: concrete::Size) -> &'static str {
    match value {
        concrete::Size::Quarter => "0.25",
        concrete::Size::Half => "0.50",
        concrete::Size::ThreeQuarter => "0.75",
        concrete::Size::Full => "1.00",
        _ => "Unknown",
    }
}

fn edit_cone_colour(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut insim_core::object::cones::ConeColour,
) {
    let _ = egui::ComboBox::from_label(label)
        .selected_text(cone_colour_name(*value))
        .show_ui(ui, |ui| {
            let _ = ui.selectable_value(value, insim_core::object::cones::ConeColour::Red, "Red");
            let _ = ui.selectable_value(value, insim_core::object::cones::ConeColour::Blue, "Blue");
            let _ = ui.selectable_value(
                value,
                insim_core::object::cones::ConeColour::Blue2,
                "Blue 2",
            );
            let _ =
                ui.selectable_value(value, insim_core::object::cones::ConeColour::Green, "Green");
            let _ = ui.selectable_value(
                value,
                insim_core::object::cones::ConeColour::Orange,
                "Orange",
            );
            let _ =
                ui.selectable_value(value, insim_core::object::cones::ConeColour::White, "White");
            let _ = ui.selectable_value(
                value,
                insim_core::object::cones::ConeColour::Yellow,
                "Yellow",
            );
        });
}

fn cone_colour_name(value: insim_core::object::cones::ConeColour) -> &'static str {
    match value {
        insim_core::object::cones::ConeColour::Red => "Red",
        insim_core::object::cones::ConeColour::Blue => "Blue",
        insim_core::object::cones::ConeColour::Blue2 => "Blue 2",
        insim_core::object::cones::ConeColour::Green => "Green",
        insim_core::object::cones::ConeColour::Orange => "Orange",
        insim_core::object::cones::ConeColour::White => "White",
        insim_core::object::cones::ConeColour::Yellow => "Yellow",
        _ => "Unknown",
    }
}

fn edit_tyre_colour(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut insim_core::object::tyres::TyreColour,
) {
    let _ = egui::ComboBox::from_label(label)
        .selected_text(tyre_colour_name(*value))
        .show_ui(ui, |ui| {
            let _ =
                ui.selectable_value(value, insim_core::object::tyres::TyreColour::Black, "Black");
            let _ =
                ui.selectable_value(value, insim_core::object::tyres::TyreColour::White, "White");
            let _ = ui.selectable_value(value, insim_core::object::tyres::TyreColour::Red, "Red");
            let _ = ui.selectable_value(value, insim_core::object::tyres::TyreColour::Blue, "Blue");
            let _ =
                ui.selectable_value(value, insim_core::object::tyres::TyreColour::Green, "Green");
            let _ = ui.selectable_value(
                value,
                insim_core::object::tyres::TyreColour::Yellow,
                "Yellow",
            );
        });
}

fn tyre_colour_name(value: insim_core::object::tyres::TyreColour) -> &'static str {
    match value {
        insim_core::object::tyres::TyreColour::Black => "Black",
        insim_core::object::tyres::TyreColour::White => "White",
        insim_core::object::tyres::TyreColour::Red => "Red",
        insim_core::object::tyres::TyreColour::Blue => "Blue",
        insim_core::object::tyres::TyreColour::Green => "Green",
        insim_core::object::tyres::TyreColour::Yellow => "Yellow",
        _ => "Unknown",
    }
}

fn edit_metal_sign_kind(ui: &mut egui::Ui, label: &str, value: &mut sign_metal::MetalSignKind) {
    let _ = egui::ComboBox::from_label(label)
        .selected_text(metal_sign_kind_name(*value))
        .show_ui(ui, |ui| {
            let _ = ui.selectable_value(value, sign_metal::MetalSignKind::KeepLeft, "Keep Left");
            let _ = ui.selectable_value(value, sign_metal::MetalSignKind::KeepRight, "Keep Right");
            let _ = ui.selectable_value(value, sign_metal::MetalSignKind::Left, "Left");
            let _ = ui.selectable_value(value, sign_metal::MetalSignKind::Right, "Right");
            let _ = ui.selectable_value(value, sign_metal::MetalSignKind::UpLeft, "Up Left");
            let _ = ui.selectable_value(value, sign_metal::MetalSignKind::UpRight, "Up Right");
            let _ = ui.selectable_value(value, sign_metal::MetalSignKind::Forward, "Forward");
            let _ = ui.selectable_value(value, sign_metal::MetalSignKind::NoEntry, "No Entry");
        });
}

fn metal_sign_kind_name(value: sign_metal::MetalSignKind) -> &'static str {
    match value {
        sign_metal::MetalSignKind::KeepLeft => "Keep Left",
        sign_metal::MetalSignKind::KeepRight => "Keep Right",
        sign_metal::MetalSignKind::Left => "Left",
        sign_metal::MetalSignKind::Right => "Right",
        sign_metal::MetalSignKind::UpLeft => "Up Left",
        sign_metal::MetalSignKind::UpRight => "Up Right",
        sign_metal::MetalSignKind::Forward => "Forward",
        sign_metal::MetalSignKind::NoEntry => "No Entry",
        _ => "Unknown",
    }
}

fn edit_speed_sign_mapping(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut sign_speed::SpeedSignMapping,
) {
    let _ = egui::ComboBox::from_label(label)
        .selected_text(speed_sign_mapping_name(*value))
        .show_ui(ui, |ui| {
            let _ = ui.selectable_value(value, sign_speed::SpeedSignMapping::Speed80Kmh, "80 km/h");
            let _ = ui.selectable_value(value, sign_speed::SpeedSignMapping::Speed50Kmh, "50 km/h");
            let _ = ui.selectable_value(value, sign_speed::SpeedSignMapping::Speed50Mph, "50 mph");
            let _ = ui.selectable_value(value, sign_speed::SpeedSignMapping::Speed40Mph, "40 mph");
        });
}

fn speed_sign_mapping_name(value: sign_speed::SpeedSignMapping) -> &'static str {
    match value {
        sign_speed::SpeedSignMapping::Speed80Kmh => "80 km/h",
        sign_speed::SpeedSignMapping::Speed50Kmh => "50 km/h",
        sign_speed::SpeedSignMapping::Speed50Mph => "50 mph",
        sign_speed::SpeedSignMapping::Speed40Mph => "40 mph",
        _ => "Unknown",
    }
}

fn edit_marker_corner_kind(ui: &mut egui::Ui, label: &str, value: &mut marker::MarkerCornerKind) {
    let _ = egui::ComboBox::from_label(label)
        .selected_text(marker_corner_kind_name(*value))
        .show_ui(ui, |ui| {
            let _ = ui.selectable_value(value, marker::MarkerCornerKind::CurveL, "Curve L");
            let _ = ui.selectable_value(value, marker::MarkerCornerKind::CurveR, "Curve R");
            let _ = ui.selectable_value(value, marker::MarkerCornerKind::L, "L");
            let _ = ui.selectable_value(value, marker::MarkerCornerKind::R, "R");
            let _ = ui.selectable_value(value, marker::MarkerCornerKind::HardL, "Hard L");
            let _ = ui.selectable_value(value, marker::MarkerCornerKind::HardR, "Hard R");
            let _ = ui.selectable_value(value, marker::MarkerCornerKind::LR, "LR");
            let _ = ui.selectable_value(value, marker::MarkerCornerKind::RL, "RL");
            let _ = ui.selectable_value(value, marker::MarkerCornerKind::SL, "SL");
            let _ = ui.selectable_value(value, marker::MarkerCornerKind::SR, "SR");
            let _ = ui.selectable_value(value, marker::MarkerCornerKind::S2L, "S2L");
            let _ = ui.selectable_value(value, marker::MarkerCornerKind::S2R, "S2R");
            let _ = ui.selectable_value(value, marker::MarkerCornerKind::UL, "UL");
            let _ = ui.selectable_value(value, marker::MarkerCornerKind::UR, "UR");
            let _ = ui.selectable_value(value, marker::MarkerCornerKind::KinkL, "Kink L");
            let _ = ui.selectable_value(value, marker::MarkerCornerKind::KinkR, "Kink R");
        });
}

fn marker_corner_kind_name(value: marker::MarkerCornerKind) -> &'static str {
    match value {
        marker::MarkerCornerKind::CurveL => "Curve L",
        marker::MarkerCornerKind::CurveR => "Curve R",
        marker::MarkerCornerKind::L => "L",
        marker::MarkerCornerKind::R => "R",
        marker::MarkerCornerKind::HardL => "Hard L",
        marker::MarkerCornerKind::HardR => "Hard R",
        marker::MarkerCornerKind::LR => "LR",
        marker::MarkerCornerKind::RL => "RL",
        marker::MarkerCornerKind::SL => "SL",
        marker::MarkerCornerKind::SR => "SR",
        marker::MarkerCornerKind::S2L => "S2L",
        marker::MarkerCornerKind::S2R => "S2R",
        marker::MarkerCornerKind::UL => "UL",
        marker::MarkerCornerKind::UR => "UR",
        marker::MarkerCornerKind::KinkL => "Kink L",
        marker::MarkerCornerKind::KinkR => "Kink R",
        _ => "Unknown",
    }
}

fn edit_marker_distance_kind(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut marker::MarkerDistanceKind,
) {
    let _ = egui::ComboBox::from_label(label)
        .selected_text(marker_distance_kind_name(*value))
        .show_ui(ui, |ui| {
            let _ = ui.selectable_value(value, marker::MarkerDistanceKind::D25, "25");
            let _ = ui.selectable_value(value, marker::MarkerDistanceKind::D50, "50");
            let _ = ui.selectable_value(value, marker::MarkerDistanceKind::D75, "75");
            let _ = ui.selectable_value(value, marker::MarkerDistanceKind::D100, "100");
            let _ = ui.selectable_value(value, marker::MarkerDistanceKind::D125, "125");
            let _ = ui.selectable_value(value, marker::MarkerDistanceKind::D150, "150");
            let _ = ui.selectable_value(value, marker::MarkerDistanceKind::D200, "200");
            let _ = ui.selectable_value(value, marker::MarkerDistanceKind::D250, "250");
        });
}

fn marker_distance_kind_name(value: marker::MarkerDistanceKind) -> &'static str {
    match value {
        marker::MarkerDistanceKind::D25 => "25",
        marker::MarkerDistanceKind::D50 => "50",
        marker::MarkerDistanceKind::D75 => "75",
        marker::MarkerDistanceKind::D100 => "100",
        marker::MarkerDistanceKind::D125 => "125",
        marker::MarkerDistanceKind::D150 => "150",
        marker::MarkerDistanceKind::D200 => "200",
        marker::MarkerDistanceKind::D250 => "250",
        _ => "Unknown",
    }
}

fn edit_paint_colour(ui: &mut egui::Ui, label: &str, value: &mut painted::PaintColour) {
    let _ = egui::ComboBox::from_label(label)
        .selected_text(paint_colour_name(*value))
        .show_ui(ui, |ui| {
            let _ = ui.selectable_value(value, painted::PaintColour::White, "White");
            let _ = ui.selectable_value(value, painted::PaintColour::Yellow, "Yellow");
        });
}

fn paint_colour_name(value: painted::PaintColour) -> &'static str {
    match value {
        painted::PaintColour::White => "White",
        painted::PaintColour::Yellow => "Yellow",
        _ => "Unknown",
    }
}

fn edit_painted_character(ui: &mut egui::Ui, label: &str, value: &mut painted::Character) {
    let mut character_text = char::from(*value).to_string();
    let _ = ui.horizontal(|ui| {
        let _ = ui.label(label);
        let response = ui.text_edit_singleline(&mut character_text);
        if response.changed()
            && let Some(character) = character_text.chars().next()
            && let Ok(next) = painted::Character::try_from(character)
        {
            *value = next;
        }
    });
}

fn edit_paint_arrow(ui: &mut egui::Ui, label: &str, value: &mut painted::Arrow) {
    let _ = egui::ComboBox::from_label(label)
        .selected_text(paint_arrow_name(*value))
        .show_ui(ui, |ui| {
            let _ = ui.selectable_value(value, painted::Arrow::Left, "Left");
            let _ = ui.selectable_value(value, painted::Arrow::Right, "Right");
            let _ = ui.selectable_value(value, painted::Arrow::StraightL, "Straight L");
            let _ = ui.selectable_value(value, painted::Arrow::StraightR, "Straight R");
            let _ = ui.selectable_value(value, painted::Arrow::CurveL, "Curve L");
            let _ = ui.selectable_value(value, painted::Arrow::CurveR, "Curve R");
            let _ = ui.selectable_value(value, painted::Arrow::StraightOn, "Straight On");
        });
}

fn paint_arrow_name(value: painted::Arrow) -> &'static str {
    match value {
        painted::Arrow::Left => "Left",
        painted::Arrow::Right => "Right",
        painted::Arrow::StraightL => "Straight L",
        painted::Arrow::StraightR => "Straight R",
        painted::Arrow::CurveL => "Curve L",
        painted::Arrow::CurveR => "Curve R",
        painted::Arrow::StraightOn => "Straight On",
        _ => "Unknown",
    }
}

fn edit_letterboard_rb_colour(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut insim_core::object::letterboard_rb::LetterboardRBColour,
) {
    let _ = egui::ComboBox::from_label(label)
        .selected_text(letterboard_rb_colour_name(*value))
        .show_ui(ui, |ui| {
            let _ = ui.selectable_value(
                value,
                insim_core::object::letterboard_rb::LetterboardRBColour::Red,
                "Red",
            );
            let _ = ui.selectable_value(
                value,
                insim_core::object::letterboard_rb::LetterboardRBColour::Blue,
                "Blue",
            );
        });
}

fn letterboard_rb_colour_name(
    value: insim_core::object::letterboard_rb::LetterboardRBColour,
) -> &'static str {
    match value {
        insim_core::object::letterboard_rb::LetterboardRBColour::Red => "Red",
        insim_core::object::letterboard_rb::LetterboardRBColour::Blue => "Blue",
        _ => "Unknown",
    }
}

fn edit_letterboard_wy_colour(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut insim_core::object::letterboard_wy::LetterboardWYColour,
) {
    let _ = egui::ComboBox::from_label(label)
        .selected_text(letterboard_wy_colour_name(*value))
        .show_ui(ui, |ui| {
            let _ = ui.selectable_value(
                value,
                insim_core::object::letterboard_wy::LetterboardWYColour::White,
                "White",
            );
            let _ = ui.selectable_value(
                value,
                insim_core::object::letterboard_wy::LetterboardWYColour::Yellow,
                "Yellow",
            );
        });
}

fn letterboard_wy_colour_name(
    value: insim_core::object::letterboard_wy::LetterboardWYColour,
) -> &'static str {
    match value {
        insim_core::object::letterboard_wy::LetterboardWYColour::White => "White",
        insim_core::object::letterboard_wy::LetterboardWYColour::Yellow => "Yellow",
        _ => "Unknown",
    }
}

fn edit_letterboard_character(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut insim_core::object::letterboard_rb::Character,
) {
    let mut character_text = char::from(*value).to_string();
    let _ = ui.horizontal(|ui| {
        let _ = ui.label(label);
        let response = ui.text_edit_singleline(&mut character_text);
        if response.changed()
            && let Some(character) = character_text.chars().next()
            && let Ok(next) = insim_core::object::letterboard_rb::Character::try_from(character)
        {
            *value = next;
        }
    });
}

fn edit_insim_checkpoint_kind(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut insim::InsimCheckpointKind,
) {
    let _ = egui::ComboBox::from_label(label)
        .selected_text(insim_checkpoint_kind_name(*value))
        .show_ui(ui, |ui| {
            let _ = ui.selectable_value(value, insim::InsimCheckpointKind::Finish, "Finish");
            let _ = ui.selectable_value(
                value,
                insim::InsimCheckpointKind::Checkpoint1,
                "Checkpoint 1",
            );
            let _ = ui.selectable_value(
                value,
                insim::InsimCheckpointKind::Checkpoint2,
                "Checkpoint 2",
            );
            let _ = ui.selectable_value(
                value,
                insim::InsimCheckpointKind::Checkpoint3,
                "Checkpoint 3",
            );
        });
}

fn insim_checkpoint_kind_name(value: insim::InsimCheckpointKind) -> &'static str {
    match value {
        insim::InsimCheckpointKind::Finish => "Finish",
        insim::InsimCheckpointKind::Checkpoint1 => "Checkpoint 1",
        insim::InsimCheckpointKind::Checkpoint2 => "Checkpoint 2",
        insim::InsimCheckpointKind::Checkpoint3 => "Checkpoint 3",
    }
}

fn edit_control_kind(ui: &mut egui::Ui, kind: &mut control::ControlKind) {
    let mut kind_index = match kind {
        control::ControlKind::Start => 0_u8,
        control::ControlKind::Finish { .. } => 1_u8,
        control::ControlKind::Checkpoint1 { .. } => 2_u8,
        control::ControlKind::Checkpoint2 { .. } => 3_u8,
        control::ControlKind::Checkpoint3 { .. } => 4_u8,
    };

    let mut half_width = match kind {
        control::ControlKind::Start => 0_u8,
        control::ControlKind::Finish { half_width }
        | control::ControlKind::Checkpoint1 { half_width }
        | control::ControlKind::Checkpoint2 { half_width }
        | control::ControlKind::Checkpoint3 { half_width } => *half_width,
    };

    let _ = egui::ComboBox::from_label("Kind")
        .selected_text(control_kind_name(kind_index))
        .show_ui(ui, |ui| {
            let _ = ui.selectable_value(&mut kind_index, 0_u8, "Start");
            let _ = ui.selectable_value(&mut kind_index, 1_u8, "Finish");
            let _ = ui.selectable_value(&mut kind_index, 2_u8, "Checkpoint 1");
            let _ = ui.selectable_value(&mut kind_index, 3_u8, "Checkpoint 2");
            let _ = ui.selectable_value(&mut kind_index, 4_u8, "Checkpoint 3");
        });

    if kind_index != 0 {
        edit_u8_range(ui, "Half Width", &mut half_width, 0, 31);
    }

    *kind = match kind_index {
        0 => control::ControlKind::Start,
        1 => control::ControlKind::Finish { half_width },
        2 => control::ControlKind::Checkpoint1 { half_width },
        3 => control::ControlKind::Checkpoint2 { half_width },
        _ => control::ControlKind::Checkpoint3 { half_width },
    };
}

fn control_kind_name(kind_index: u8) -> &'static str {
    match kind_index {
        0 => "Start",
        1 => "Finish",
        2 => "Checkpoint 1",
        3 => "Checkpoint 2",
        _ => "Checkpoint 3",
    }
}

fn edit_marshal_kind(ui: &mut egui::Ui, label: &str, value: &mut marshal::MarshalKind) {
    let _ = egui::ComboBox::from_label(label)
        .selected_text(marshal_kind_name(*value))
        .show_ui(ui, |ui| {
            let _ = ui.selectable_value(value, marshal::MarshalKind::Standing, "Standing");
            let _ = ui.selectable_value(value, marshal::MarshalKind::Left, "Left");
            let _ = ui.selectable_value(value, marshal::MarshalKind::Right, "Right");
        });
}

fn marshal_kind_name(value: marshal::MarshalKind) -> &'static str {
    match value {
        marshal::MarshalKind::Standing => "Standing",
        marshal::MarshalKind::Left => "Left",
        marshal::MarshalKind::Right => "Right",
        _ => "Unknown",
    }
}
