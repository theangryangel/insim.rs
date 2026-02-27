use insim::{
    core::{
        heading::Heading,
        object::{painted, ObjectCoordinate},
    },
    insim::ObjectInfo,
};

pub fn build(text: &str, anchor: ObjectCoordinate, heading: Heading) -> Vec<ObjectInfo> {
    const SPACING_RAW_UNITS: i32 = 16;
    const PADDING_SLOTS: i32 = 2;

    let radians = heading.to_radians();
    let right_x = radians.cos();
    let right_y = radians.sin();
    let anchor_x = f64::from(anchor.x);
    let anchor_y = f64::from(anchor.y);

    text.chars()
        .enumerate()
        .filter_map(|(index, ch)| {
            let character = painted::Character::try_from(ch).ok()?;
            let slot = i32::try_from(index).ok()?.saturating_add(PADDING_SLOTS);
            let offset = f64::from(slot.saturating_mul(SPACING_RAW_UNITS));

            let x = crate::clamp_i16((anchor_x + (right_x * offset)).round() as i32);
            let y = crate::clamp_i16((anchor_y + (right_y * offset)).round() as i32);

            Some(ObjectInfo::PaintLetters(painted::Letters {
                xyz: ObjectCoordinate { x, y, z: anchor.z },
                colour: painted::PaintColour::Yellow,
                character,
                heading,
                floating: false,
            }))
        })
        .collect()
}
