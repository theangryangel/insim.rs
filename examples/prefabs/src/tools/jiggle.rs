use insim::{core::heading::Heading, insim::ObjectInfo};
use noise::{NoiseFn, Perlin};
use rand_distr::{Distribution, Normal};

// scale determines how "large" the spatial patches are.
// 0.1 means the wobble changes slowly over 10 meters.
const SPATIAL_SCALE: f64 = 0.1;

pub fn jiggle(
    selection: &[ObjectInfo],
    spatial_intensity: f64,    // e.g., 5.0 degrees
    individual_intensity: f64, // e.g., 1.5 degrees
) -> Vec<ObjectInfo> {
    let mut output = Vec::with_capacity(selection.len());

    let seed = rand::random();
    let perlin = Perlin::new(seed);
    let normal = Normal::new(0.0, individual_intensity).unwrap();
    let mut rng = rand::rng();

    for obj in selection {
        let mut new_obj = obj.clone();

        let pos = new_obj.position().to_dvec3_metres();
        let current_rads = new_obj.heading().map(|h| h.to_radians()).unwrap_or(0.0);

        // 1. Get the Spatial "Trend" (Perlin)
        let trend = perlin.get([pos.x * SPATIAL_SCALE, pos.y * SPATIAL_SCALE]);
        let spatial_jiggle = trend * spatial_intensity.to_radians();

        // 2. Get the Individual "Bump" (Gaussian)
        let individual_jiggle = normal.sample(&mut rng).to_radians();

        // 3. Combine and Apply
        let final_rads = current_rads + spatial_jiggle + individual_jiggle;
        if let Some(heading) = new_obj.heading_mut() {
            *heading = Heading::from_radians(final_rads);
        }

        output.push(new_obj);
    }

    output
}
