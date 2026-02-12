use crate::{heading::Heading, object::*};

fn raw_flags(flags: u8) -> Raw {
    Raw {
        index: 0,
        xyz: ObjectCoordinate::new(0, 0, 0),
        flags,
        heading: 0,
    }
}

fn raw_with(xyz: ObjectCoordinate, flags: u8, heading: Heading) -> Raw {
    Raw {
        index: 0,
        xyz,
        flags,
        heading: heading.to_objectinfo_wire(),
    }
}

fn raw_with_heading_u8(xyz: ObjectCoordinate, flags: u8, heading: u8) -> Raw {
    Raw {
        index: 0,
        xyz,
        flags,
        heading,
    }
}

#[test]
fn test_armco1_basic_creation() {
    let coord = ObjectCoordinate::new(100, 200, 50);
    let heading = Heading::from_degrees(45.0);
    let armco = armco::Armco {
        xyz: coord,
        heading,
        colour: 2,
        mapping: 3,
        floating: false,
    };

    assert_eq!(armco.xyz.x, 100);
    assert_eq!(armco.xyz.y, 200);
    assert_eq!(armco.xyz.z, 50);
    assert_eq!(armco.colour, 2);
    assert_eq!(armco.mapping, 3);
    assert!(!armco.floating);
}

#[test]
fn test_armco1_floating() {
    let armco = armco::Armco {
        xyz: ObjectCoordinate::new(0, 0, 0),
        heading: Heading::from_degrees(0.0),
        colour: 1,
        mapping: 1,
        floating: true,
    };

    assert!(armco.floating);
}

#[test]
fn test_armco1_flags_conversion() {
    let armco = armco::Armco {
        xyz: ObjectCoordinate::new(0, 0, 0),
        heading: Heading::from_degrees(0.0),
        colour: 3,
        mapping: 5,
        floating: true,
    };

    let flags = armco.flags();
    // colour (3 bits): 0-7
    // mapping (4 bits at bits 3-6): 0-15
    // floating (bit 7): 0-1
    assert_eq!(raw_flags(flags).raw_colour(), 3);
    assert_eq!(raw_flags(flags).raw_mapping(), 5);
    assert!(raw_flags(flags).raw_floating());
}

#[test]
fn test_armco1_colour_boundary() {
    let armco = armco::Armco {
        xyz: ObjectCoordinate::new(0, 0, 0),
        heading: Heading::from_degrees(0.0),
        colour: 7, // Max 3-bit value
        mapping: 0,
        floating: false,
    };

    assert_eq!(armco.colour, 7);
}

#[test]
fn test_armco1_mapping_boundary() {
    let armco = armco::Armco {
        xyz: ObjectCoordinate::new(0, 0, 0),
        heading: Heading::from_degrees(0.0),
        colour: 0,
        mapping: 15, // Max 4-bit value
        floating: false,
    };

    assert_eq!(armco.mapping, 15);
}

#[test]
fn test_chalk_basic_creation() {
    let coord = ObjectCoordinate::new(150, 250, 30);
    let heading = Heading::from_degrees(90.0);
    let chalk = chalk::Chalk {
        xyz: coord,
        colour: chalk::ChalkColour::Red,
        heading,
        floating: false,
    };

    assert_eq!(chalk.xyz.x, 150);
    assert_eq!(chalk.xyz.y, 250);
    assert_eq!(chalk.colour, chalk::ChalkColour::Red);
    assert!(!chalk.floating);
}

#[test]
fn test_chalk_all_colours() {
    let colours = vec![
        chalk::ChalkColour::White,
        chalk::ChalkColour::Red,
        chalk::ChalkColour::Blue,
        chalk::ChalkColour::Yellow,
    ];

    for colour in colours {
        let chalk = chalk::Chalk {
            xyz: ObjectCoordinate::new(0, 0, 0),
            colour,
            heading: Heading::from_degrees(0.0),
            floating: false,
        };
        assert_eq!(chalk.colour, colour);
    }
}

#[test]
fn test_chalk_colour_conversion() {
    let chalk_white = chalk::Chalk {
        xyz: ObjectCoordinate::new(0, 0, 0),
        colour: chalk::ChalkColour::White,
        heading: Heading::from_degrees(0.0),
        floating: false,
    };

    let flags = chalk_white.flags();
    assert_eq!(raw_flags(flags).raw_colour(), 0);
}

#[test]
fn test_chalk_floating_flag() {
    let chalk = chalk::Chalk {
        xyz: ObjectCoordinate::new(0, 0, 0),
        colour: chalk::ChalkColour::Blue,
        heading: Heading::from_degrees(0.0),
        floating: true,
    };

    let flags = chalk.flags();
    assert!(raw_flags(flags).raw_floating());
}

#[test]
fn test_chalk_from_flags() {
    let flags = raw_flags(0x82); // floating + colour 2 (blue)
    let heading = Heading::from_degrees(45.0);
    let result = chalk::Chalk::new(raw_with(
        ObjectCoordinate::new(100, 100, 10),
        flags.flags,
        heading,
    ));

    assert!(result.is_ok());
    let chalk = result.unwrap();
    assert_eq!(chalk.colour, chalk::ChalkColour::Blue);
    assert!(chalk.floating);
}

#[test]
fn test_insim_checkpoint_finish() {
    let checkpoint = insim::InsimCheckpoint {
        xyz: ObjectCoordinate::new(500, 500, 0),
        kind: insim::InsimCheckpointKind::Finish,
        heading: Heading::from_degrees(0.0),
        floating: false,
    };

    assert_eq!(checkpoint.kind, insim::InsimCheckpointKind::Finish);
}

#[test]
fn test_insim_checkpoint_all_kinds() {
    let kinds = vec![
        insim::InsimCheckpointKind::Finish,
        insim::InsimCheckpointKind::Checkpoint1,
        insim::InsimCheckpointKind::Checkpoint2,
        insim::InsimCheckpointKind::Checkpoint3,
    ];

    for kind in kinds {
        let checkpoint = insim::InsimCheckpoint {
            xyz: ObjectCoordinate::new(0, 0, 0),
            kind,
            heading: Heading::from_degrees(0.0),
            floating: false,
        };
        assert_eq!(checkpoint.kind, kind);
    }
}

#[test]
fn test_insim_checkpoint_floating() {
    let checkpoint = insim::InsimCheckpoint {
        xyz: ObjectCoordinate::new(0, 0, 0),
        kind: insim::InsimCheckpointKind::Checkpoint1,
        heading: Heading::from_degrees(0.0),
        floating: true,
    };

    assert!(checkpoint.floating);
    let flags = checkpoint.flags();
    assert!(raw_flags(flags).raw_floating());
}

#[test]
fn test_insim_checkpoint_from_flags() {
    let flags = raw_flags(0x82); // floating + kind checkpoint2
    let heading = Heading::from_degrees(90.0);
    let result = insim::InsimCheckpoint::new(raw_with(
        ObjectCoordinate::new(200, 300, 5),
        flags.flags,
        heading,
    ));

    assert!(result.is_ok());
    let checkpoint = result.unwrap();
    assert_eq!(checkpoint.kind, insim::InsimCheckpointKind::Checkpoint2);
    assert!(checkpoint.floating);
}

#[test]
fn test_insim_circle_basic() {
    let circle = insim::InsimCircle {
        xyz: ObjectCoordinate::new(1000, 1000, 50),
        index: 1,
        floating: false,
    };

    assert_eq!(circle.index, 1);
    assert!(!circle.floating);
}

#[test]
fn test_insim_circle_various_indices() {
    for index in 0u8..=255 {
        let circle = insim::InsimCircle {
            xyz: ObjectCoordinate::new(0, 0, 0),
            index,
            floating: false,
        };
        assert_eq!(circle.index, index);
    }
}

#[test]
fn test_insim_circle_floating() {
    let circle = insim::InsimCircle {
        xyz: ObjectCoordinate::new(500, 500, 25),
        index: 5,
        floating: true,
    };

    assert!(circle.floating);
    let flags = circle.flags();
    assert!(raw_flags(flags).raw_floating());
}

#[test]
fn test_insim_circle_from_flags() {
    let flags = raw_flags(0x80); // floating only
    let result = insim::InsimCircle::new(raw_with_heading_u8(
        ObjectCoordinate::new(100, 100, 10),
        flags.flags,
        42,
    ));

    assert!(result.is_ok());
    let circle = result.unwrap();
    assert_eq!(circle.index, 42);
    assert!(circle.floating);
}

#[test]
fn test_paint_letters_basic() {
    let letters = painted::Letters {
        xyz: ObjectCoordinate::new(300, 400, 20),
        colour: painted::PaintColour::White,
        character: painted::Character::A,
        heading: Heading::from_degrees(0.0),
        floating: false,
    };

    assert_eq!(letters.colour, painted::PaintColour::White);
    assert_eq!(letters.character, painted::Character::A);
}

#[test]
fn test_paint_letters_various_characters() {
    let characters = vec![
        painted::Character::A,
        painted::Character::Z,
        painted::Character::Zero,
        painted::Character::Nine,
        painted::Character::Dot,
    ];

    for character in characters {
        let letters = painted::Letters {
            xyz: ObjectCoordinate::new(0, 0, 0),
            colour: painted::PaintColour::White,
            character,
            heading: Heading::from_degrees(0.0),
            floating: false,
        };
        assert_eq!(letters.character, character);
    }
}

#[test]
fn test_paint_letters_colours() {
    let colours = vec![painted::PaintColour::White, painted::PaintColour::Yellow];

    for colour in colours {
        let letters = painted::Letters {
            xyz: ObjectCoordinate::new(0, 0, 0),
            colour,
            character: painted::Character::A,
            heading: Heading::from_degrees(0.0),
            floating: false,
        };
        assert_eq!(letters.colour, colour);
    }
}

#[test]
fn test_paint_letters_character_conversion_to_char() {
    let character = painted::Character::A;
    let ch: char = character.into();
    assert_eq!(ch, 'A');

    let character = painted::Character::Zero;
    let ch: char = character.into();
    assert_eq!(ch, '0');
}

#[test]
fn test_paint_letters_floating() {
    let letters = painted::Letters {
        xyz: ObjectCoordinate::new(0, 0, 0),
        colour: painted::PaintColour::Yellow,
        character: painted::Character::B,
        heading: Heading::from_degrees(45.0),
        floating: true,
    };

    assert!(letters.floating);
}

#[test]
fn test_letterboard_rb_basic() {
    let board = letterboard_rb::LetterboardRB {
        xyz: ObjectCoordinate::new(600, 700, 40),
        colour: letterboard_rb::LetterboardRBColour::Red,
        heading: Heading::from_degrees(0.0),
        character: letterboard_rb::Character::A,
        floating: false,
    };

    assert_eq!(board.colour, letterboard_rb::LetterboardRBColour::Red);
    assert_eq!(board.character, letterboard_rb::Character::A);
}

#[test]
fn test_letterboard_rb_colours() {
    let board_red = letterboard_rb::LetterboardRB {
        xyz: ObjectCoordinate::new(0, 0, 0),
        colour: letterboard_rb::LetterboardRBColour::Red,
        heading: Heading::from_degrees(0.0),
        character: letterboard_rb::Character::A,
        floating: false,
    };

    let board_blue = letterboard_rb::LetterboardRB {
        xyz: ObjectCoordinate::new(0, 0, 0),
        colour: letterboard_rb::LetterboardRBColour::Blue,
        heading: Heading::from_degrees(0.0),
        character: letterboard_rb::Character::A,
        floating: false,
    };

    assert_eq!(board_red.colour, letterboard_rb::LetterboardRBColour::Red);
    assert_eq!(board_blue.colour, letterboard_rb::LetterboardRBColour::Blue);
}

#[test]
fn test_letterboard_rb_blank_character() {
    let board = letterboard_rb::LetterboardRB {
        xyz: ObjectCoordinate::new(0, 0, 0),
        colour: letterboard_rb::LetterboardRBColour::Red,
        heading: Heading::from_degrees(0.0),
        character: letterboard_rb::Character::Blank,
        floating: false,
    };

    assert_eq!(board.character, letterboard_rb::Character::Blank);
    let ch: char = board.character.into();
    assert_eq!(ch, ' ');
}

#[test]
fn test_letterboard_rb_floating() {
    let board = letterboard_rb::LetterboardRB {
        xyz: ObjectCoordinate::new(0, 0, 0),
        colour: letterboard_rb::LetterboardRBColour::Blue,
        heading: Heading::from_degrees(90.0),
        character: letterboard_rb::Character::Z,
        floating: true,
    };

    assert!(board.floating);
    let flags = board.flags();
    assert!(raw_flags(flags).raw_floating());
}

#[test]
fn test_letterboard_rb_from_flags() {
    let flags = raw_flags(0x81); // floating + colour blue + character A
    let heading = Heading::from_degrees(0.0);
    let result = letterboard_rb::LetterboardRB::new(raw_with(
        ObjectCoordinate::new(0, 0, 0),
        flags.flags,
        heading,
    ));

    assert!(result.is_ok());
    let board = result.unwrap();
    assert!(board.floating);
    assert_eq!(board.colour, letterboard_rb::LetterboardRBColour::Blue);
}

#[test]
fn test_tyre_stack2_basic() {
    let tyre = tyres::Tyres {
        xyz: ObjectCoordinate::new(200, 300, 0),
        colour: tyres::TyreColour::Black,
        heading: Heading::from_degrees(0.0),
        floating: false,
    };

    assert_eq!(tyre.colour, tyres::TyreColour::Black);
}

#[test]
fn test_tyre_stack2_all_colours() {
    let colours = vec![
        tyres::TyreColour::Black,
        tyres::TyreColour::White,
        tyres::TyreColour::Red,
        tyres::TyreColour::Blue,
        tyres::TyreColour::Green,
        tyres::TyreColour::Yellow,
    ];

    for colour in colours {
        let tyre = tyres::Tyres {
            xyz: ObjectCoordinate::new(0, 0, 0),
            colour,
            heading: Heading::from_degrees(0.0),
            floating: false,
        };
        assert_eq!(tyre.colour, colour);
    }
}

#[test]
fn test_tyre_stack2_flags_conversion() {
    let tyre = tyres::Tyres {
        xyz: ObjectCoordinate::new(0, 0, 0),
        colour: tyres::TyreColour::Red,
        heading: Heading::from_degrees(0.0),
        floating: false,
    };

    let flags = tyre.flags();
    assert_eq!(raw_flags(flags).raw_colour(), 2); // Red is value 2
}

#[test]
fn test_tyre_stack2_floating() {
    let tyre = tyres::Tyres {
        xyz: ObjectCoordinate::new(100, 100, 0),
        colour: tyres::TyreColour::Yellow,
        heading: Heading::from_degrees(45.0),
        floating: true,
    };

    assert!(tyre.floating);
    let flags = tyre.flags();
    assert!(raw_flags(flags).raw_floating());
}

#[test]
fn test_tyre_stack2_from_flags() {
    let flags = raw_flags(0x83); // floating + colour green (4)
    let heading = Heading::from_degrees(180.0);
    let result = tyres::Tyres::new(raw_with(
        ObjectCoordinate::new(50, 50, 0),
        flags.flags,
        heading,
    ));

    assert!(result.is_ok());
    let tyre = result.unwrap();
    assert!(tyre.floating);
}

#[test]
fn test_post_basic() {
    let post = post::Post {
        xyz: ObjectCoordinate::new(400, 500, 100),
        heading: Heading::from_degrees(0.0),
        colour: post::PostColour::Green,
        mapping: 1,
        floating: false,
    };

    assert_eq!(post.colour, post::PostColour::Green);
    assert_eq!(post.mapping, 1);
}

#[test]
fn test_post_all_colours() {
    let colours = vec![
        post::PostColour::Green,
        post::PostColour::Orange,
        post::PostColour::Red,
        post::PostColour::White,
        post::PostColour::Blue,
        post::PostColour::Yellow,
        post::PostColour::LightBlue,
    ];

    for colour in colours {
        let post = post::Post {
            xyz: ObjectCoordinate::new(0, 0, 0),
            heading: Heading::from_degrees(0.0),
            colour,
            mapping: 0,
            floating: false,
        };
        assert_eq!(post.colour, colour);
    }
}

#[test]
fn test_post_mapping_range() {
    for mapping in 0u8..=15 {
        let post = post::Post {
            xyz: ObjectCoordinate::new(0, 0, 0),
            heading: Heading::from_degrees(0.0),
            colour: post::PostColour::Red,
            mapping,
            floating: false,
        };
        assert_eq!(post.mapping, mapping);
    }
}

#[test]
fn test_post_floating() {
    let post = post::Post {
        xyz: ObjectCoordinate::new(0, 0, 0),
        heading: Heading::from_degrees(0.0),
        colour: post::PostColour::Blue,
        mapping: 5,
        floating: true,
    };

    assert!(post.floating);
    let flags = post.flags();
    assert!(raw_flags(flags).raw_floating());
}

#[test]
fn test_post_from_flags() {
    let flags = raw_flags(0x8A); // floating + colour 2 (red) + mapping 1
    let heading = Heading::from_degrees(270.0);
    let result = post::Post::new(raw_with(
        ObjectCoordinate::new(0, 0, 0),
        flags.flags,
        heading,
    ));

    assert!(result.is_ok());
    let post = result.unwrap();
    assert!(post.floating);
    assert_eq!(post.colour, post::PostColour::Red);
    assert_eq!(post.mapping, 1);
}

#[test]
fn test_start_lights_basic() {
    let lights = start_lights::StartLights {
        xyz: ObjectCoordinate::new(0, 0, 200),
        heading: Heading::from_degrees(0.0),
        identifier: 0,
        floating: false,
    };

    assert_eq!(lights.identifier, 0);
}

#[test]
fn test_start_lights_identifier_range() {
    for id in 0u8..=63 {
        let lights = start_lights::StartLights {
            xyz: ObjectCoordinate::new(0, 0, 0),
            heading: Heading::from_degrees(0.0),
            identifier: id,
            floating: false,
        };
        assert_eq!(lights.identifier, id);
    }
}

#[test]
fn test_start_lights_flags_conversion() {
    let lights = start_lights::StartLights {
        xyz: ObjectCoordinate::new(0, 0, 0),
        heading: Heading::from_degrees(0.0),
        identifier: 42,
        floating: false,
    };

    let flags = lights.flags();
    assert_eq!(flags & 0x3F, 42); // Lower 6 bits should be identifier
}

#[test]
fn test_start_lights_floating() {
    let lights = start_lights::StartLights {
        xyz: ObjectCoordinate::new(100, 100, 150),
        heading: Heading::from_degrees(90.0),
        identifier: 15,
        floating: true,
    };

    assert!(lights.floating);
    let flags = lights.flags();
    assert!(raw_flags(flags).raw_floating());
}

#[test]
fn test_start_lights_from_flags() {
    let flags = raw_flags(0xAA); // floating + identifier 42 (0x2A)
    let heading = Heading::from_degrees(45.0);
    let result = start_lights::StartLights::new(raw_with(
        ObjectCoordinate::new(0, 0, 0),
        flags.flags,
        heading,
    ));

    assert!(result.is_ok());
    let lights = result.unwrap();
    assert!(lights.floating);
    assert_eq!(lights.identifier, 42);
}

#[test]
fn test_vehicle_van_basic() {
    let van = vehicle_van::VehicleVan {
        xyz: ObjectCoordinate::new(800, 900, 50),
        heading: Heading::from_degrees(0.0),
        colour: vehicle_van::VehicleVanColour::White,
        mapping: 2,
        floating: false,
    };

    assert_eq!(van.colour, vehicle_van::VehicleVanColour::White);
    assert_eq!(van.mapping, 2);
}

#[test]
fn test_vehicle_van_all_colours() {
    let colours = vec![
        vehicle_van::VehicleVanColour::White,
        vehicle_van::VehicleVanColour::Red,
        vehicle_van::VehicleVanColour::Blue,
        vehicle_van::VehicleVanColour::Green,
        vehicle_van::VehicleVanColour::Yellow,
        vehicle_van::VehicleVanColour::Turquoise,
        vehicle_van::VehicleVanColour::Black,
    ];

    for colour in colours {
        let van = vehicle_van::VehicleVan {
            xyz: ObjectCoordinate::new(0, 0, 0),
            heading: Heading::from_degrees(0.0),
            colour,
            mapping: 0,
            floating: false,
        };
        assert_eq!(van.colour, colour);
    }
}

#[test]
fn test_vehicle_van_mapping_range() {
    for mapping in 0u8..=15 {
        let van = vehicle_van::VehicleVan {
            xyz: ObjectCoordinate::new(0, 0, 0),
            heading: Heading::from_degrees(0.0),
            colour: vehicle_van::VehicleVanColour::Blue,
            mapping,
            floating: false,
        };
        assert_eq!(van.mapping, mapping);
    }
}

#[test]
fn test_vehicle_van_floating() {
    let van = vehicle_van::VehicleVan {
        xyz: ObjectCoordinate::new(0, 0, 0),
        heading: Heading::from_degrees(180.0),
        colour: vehicle_van::VehicleVanColour::Red,
        mapping: 7,
        floating: true,
    };

    assert!(van.floating);
    let flags = van.flags();
    assert!(raw_flags(flags).raw_floating());
}

#[test]
fn test_vehicle_van_from_flags() {
    let flags = raw_flags(0x89); // floating + colour 1 (red) + mapping 1
    let heading = Heading::from_degrees(315.0);
    let result = vehicle_van::VehicleVan::new(raw_with(
        ObjectCoordinate::new(0, 0, 0),
        flags.flags,
        heading,
    ));

    assert!(result.is_ok());
    let van = result.unwrap();
    assert!(van.floating);
    assert_eq!(van.colour, vehicle_van::VehicleVanColour::Red);
    assert_eq!(van.mapping, 1);
}

#[test]
fn test_coordinate_metre_conversions() {
    let coord = ObjectCoordinate::new(160, 320, 4); // 10m x, 20m y, 1m z

    assert!((coord.x_metres() - 10.0).abs() < 0.0001);
    assert!((coord.y_metres() - 20.0).abs() < 0.0001);
    assert!((coord.z_metres() - 1.0).abs() < 0.0001);
}

#[test]
fn test_coordinate_xyz_metres() {
    let coord = ObjectCoordinate::new(16, 32, 8); // 1m x, 2m y, 2m z
    let (x, y, z) = coord.xyz_metres();

    assert!((x - 1.0).abs() < 0.0001);
    assert!((y - 2.0).abs() < 0.0001);
    assert!((z - 2.0).abs() < 0.0001);
}

#[test]
fn test_coordinate_negative_values() {
    let coord = ObjectCoordinate::new(-160, -320, 0);

    assert!((coord.x_metres() - (-10.0)).abs() < 0.0001);
    assert!((coord.y_metres() - (-20.0)).abs() < 0.0001);
}

#[test]
fn test_coordinate_equality() {
    let coord1 = ObjectCoordinate::new(100, 200, 50);
    let coord2 = ObjectCoordinate::new(100, 200, 50);

    assert_eq!(coord1, coord2);
}
