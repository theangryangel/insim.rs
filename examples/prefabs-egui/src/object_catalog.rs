//! Placeable object catalog.

use insim_core::object::ObjectInfo;

/// High-level object categories for the place library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObjectCategory {
    /// Control and logic objects.
    TrackLogic,
    /// Chalk drawing objects.
    Chalk,
    /// Painted letters and arrows.
    Painted,
    /// Cone objects.
    Cones,
    /// Tyre stack objects.
    Tyres,
    /// Marker objects.
    Markers,
    /// Letterboard objects.
    Letterboards,
    /// Armco / barrier style objects.
    Boundaries,
    /// Ramp objects.
    Ramps,
    /// Vehicle props.
    Vehicles,
    /// Speed hump objects.
    SpeedHumps,
    /// General props and furniture.
    Props,
    /// Start / pit placement objects.
    StartAndPit,
    /// Lighting objects.
    Lights,
    /// Signage objects.
    Signs,
    /// Concrete construction objects.
    Concrete,
}

impl ObjectCategory {
    /// Human-readable category label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::TrackLogic => "Track Logic",
            Self::Chalk => "Chalk",
            Self::Painted => "Painted",
            Self::Cones => "Cones",
            Self::Tyres => "Tyres",
            Self::Markers => "Markers",
            Self::Letterboards => "Letterboards",
            Self::Boundaries => "Boundaries",
            Self::Ramps => "Ramps",
            Self::Vehicles => "Vehicles",
            Self::SpeedHumps => "Speed Humps",
            Self::Props => "Props",
            Self::StartAndPit => "Start and Pit",
            Self::Lights => "Lights",
            Self::Signs => "Signs",
            Self::Concrete => "Concrete",
        }
    }

    /// Short category icon label.
    pub const fn icon(self) -> &'static str {
        match self {
            Self::TrackLogic => "[TL]",
            Self::Chalk => "[CH]",
            Self::Painted => "[PT]",
            Self::Cones => "[CN]",
            Self::Tyres => "[TY]",
            Self::Markers => "[MK]",
            Self::Letterboards => "[LB]",
            Self::Boundaries => "[BD]",
            Self::Ramps => "[RP]",
            Self::Vehicles => "[VH]",
            Self::SpeedHumps => "[SH]",
            Self::Props => "[PR]",
            Self::StartAndPit => "[SP]",
            Self::Lights => "[LG]",
            Self::Signs => "[SG]",
            Self::Concrete => "[CC]",
        }
    }
}

/// Category display order in the place library.
pub const CATEGORY_ORDER: &[ObjectCategory] = &[
    ObjectCategory::TrackLogic,
    ObjectCategory::Chalk,
    ObjectCategory::Painted,
    ObjectCategory::Cones,
    ObjectCategory::Tyres,
    ObjectCategory::Markers,
    ObjectCategory::Letterboards,
    ObjectCategory::Boundaries,
    ObjectCategory::Ramps,
    ObjectCategory::Vehicles,
    ObjectCategory::SpeedHumps,
    ObjectCategory::Props,
    ObjectCategory::StartAndPit,
    ObjectCategory::Lights,
    ObjectCategory::Signs,
    ObjectCategory::Concrete,
];

macro_rules! define_object_catalog {
    ($( $kind:ident => ($name:literal, $category:ident), )+ ) => {
        /// A concrete placeable object kind.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum ObjectCatalogKind {
            $(
                /// Placeable object kind.
                $kind,
            )+
        }

        impl ObjectCatalogKind {
            /// Creates a typed object with default variant-specific fields.
            pub fn create_default(self) -> ObjectInfo {
                match self {
                    $(
                        Self::$kind => ObjectInfo::$kind(Default::default()),
                    )+
                }
            }
        }

        /// Object catalog entry.
        #[derive(Debug, Clone, Copy)]
        pub struct ObjectCatalogEntry {
            /// Object kind.
            pub kind: ObjectCatalogKind,
            /// Human readable name.
            pub name: &'static str,
            /// Category grouping.
            pub category: ObjectCategory,
        }

        /// All known placeable object kinds supported by `ObjectInfo`.
        pub const ALL_OBJECTS: &[ObjectCatalogEntry] = &[
            $(
                ObjectCatalogEntry {
                    kind: ObjectCatalogKind::$kind,
                    name: $name,
                    category: ObjectCategory::$category,
                },
            )+
        ];

        /// Finds a catalog entry by object kind.
        pub fn find_entry(kind: ObjectCatalogKind) -> Option<ObjectCatalogEntry> {
            ALL_OBJECTS
                .iter()
                .copied()
                .find(|entry| entry.kind == kind)
        }
    };
}

define_object_catalog! {
    Control => ("Control", TrackLogic),
    ChalkLine => ("Chalk Line", Chalk),
    ChalkLine2 => ("Chalk Line 2", Chalk),
    ChalkAhead => ("Chalk Ahead", Chalk),
    ChalkAhead2 => ("Chalk Ahead 2", Chalk),
    ChalkLeft => ("Chalk Left", Chalk),
    ChalkLeft2 => ("Chalk Left 2", Chalk),
    ChalkLeft3 => ("Chalk Left 3", Chalk),
    ChalkRight => ("Chalk Right", Chalk),
    ChalkRight2 => ("Chalk Right 2", Chalk),
    ChalkRight3 => ("Chalk Right 3", Chalk),
    PaintLetters => ("Painted Letters", Painted),
    PaintArrows => ("Painted Arrows", Painted),
    Cone1 => ("Cone 1", Cones),
    Cone2 => ("Cone 2", Cones),
    ConeTall1 => ("Cone Tall 1", Cones),
    ConeTall2 => ("Cone Tall 2", Cones),
    ConePointer => ("Cone Pointer", Cones),
    TyreSingle => ("Tyre Single", Tyres),
    TyreStack2 => ("Tyre Stack 2", Tyres),
    TyreStack3 => ("Tyre Stack 3", Tyres),
    TyreStack4 => ("Tyre Stack 4", Tyres),
    TyreSingleBig => ("Tyre Single Big", Tyres),
    TyreStack2Big => ("Tyre Stack 2 Big", Tyres),
    TyreStack3Big => ("Tyre Stack 3 Big", Tyres),
    TyreStack4Big => ("Tyre Stack 4 Big", Tyres),
    MarkerCorner => ("Marker Corner", Markers),
    MarkerDistance => ("Marker Distance", Markers),
    LetterboardWY => ("Letterboard WY", Letterboards),
    LetterboardRB => ("Letterboard RB", Letterboards),
    Armco1 => ("Armco 1", Boundaries),
    Armco3 => ("Armco 3", Boundaries),
    Armco5 => ("Armco 5", Boundaries),
    BarrierLong => ("Barrier Long", Boundaries),
    BarrierRed => ("Barrier Red", Boundaries),
    BarrierWhite => ("Barrier White", Boundaries),
    Banner => ("Banner", Boundaries),
    Ramp1 => ("Ramp 1", Ramps),
    Ramp2 => ("Ramp 2", Ramps),
    VehicleSUV => ("Vehicle SUV", Vehicles),
    VehicleVan => ("Vehicle Van", Vehicles),
    VehicleTruck => ("Vehicle Truck", Vehicles),
    VehicleAmbulance => ("Vehicle Ambulance", Vehicles),
    SpeedHump10M => ("Speed Hump 10m", SpeedHumps),
    SpeedHump6M => ("Speed Hump 6m", SpeedHumps),
    SpeedHump2M => ("Speed Hump 2m", SpeedHumps),
    SpeedHump1M => ("Speed Hump 1m", SpeedHumps),
    Kerb => ("Kerb", Props),
    Post => ("Post", Props),
    Marquee => ("Marquee", Props),
    Bale => ("Bale", Props),
    Bin1 => ("Bin 1", Props),
    Bin2 => ("Bin 2", Props),
    Railing1 => ("Railing 1", Boundaries),
    Railing2 => ("Railing 2", Boundaries),
    StartLights1 => ("Start Lights 1", Lights),
    StartLights2 => ("Start Lights 2", Lights),
    StartLights3 => ("Start Lights 3", Lights),
    SignMetal => ("Sign Metal", Signs),
    ChevronLeft => ("Chevron Left", Signs),
    ChevronRight => ("Chevron Right", Signs),
    SignSpeed => ("Sign Speed", Signs),
    ConcreteSlab => ("Concrete Slab", Concrete),
    ConcreteRamp => ("Concrete Ramp", Concrete),
    ConcreteWall => ("Concrete Wall", Concrete),
    ConcretePillar => ("Concrete Pillar", Concrete),
    ConcreteSlabWall => ("Concrete Slab Wall", Concrete),
    ConcreteRampWall => ("Concrete Ramp Wall", Concrete),
    ConcreteShortSlabWall => ("Concrete Short Slab Wall", Concrete),
    ConcreteWedge => ("Concrete Wedge", Concrete),
    StartPosition => ("Start Position", StartAndPit),
    PitStartPoint => ("Pit Start Point", StartAndPit),
    PitStopBox => ("Pit Stop Box", StartAndPit),
    Marshal => ("Marshal", TrackLogic),
    InsimCheckpoint => ("InSim Checkpoint", TrackLogic),
    InsimCircle => ("InSim Circle", TrackLogic),
    RestrictedArea => ("Restricted Area", TrackLogic),
    RouteChecker => ("Route Checker", TrackLogic),
}

impl Default for ObjectCatalogKind {
    fn default() -> Self {
        Self::PaintLetters
    }
}
