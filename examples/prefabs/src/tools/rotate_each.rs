use anyhow::{Result, ensure};
use insim::{core::heading::Heading, insim::ObjectInfo};

/// Rotates each object's heading in place by `degrees`, without moving positions.
pub fn build(selection: &[ObjectInfo], degrees: f64) -> Result<Vec<ObjectInfo>> {
    ensure!(
        !selection.is_empty(),
        "rotate each skipped: selection is empty"
    );
    ensure!(
        degrees.is_finite(),
        "rotate each skipped: degrees must be finite"
    );

    let radians = degrees.to_radians();

    let rotated = selection
        .iter()
        .cloned()
        .map(|mut obj| {
            if let Some(heading) = obj.heading_mut() {
                *heading = Heading::from_radians(heading.to_radians() + radians);
            }
            obj
        })
        .collect();

    Ok(rotated)
}
