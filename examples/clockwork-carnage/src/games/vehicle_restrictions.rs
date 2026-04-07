use insim::{
    builder::InsimTask,
    core::vehicle::Vehicle,
    identifiers::ConnectionId,
    insim::{Mal, Plc, PlcAllowedCarsSet},
};
use insim_extras::scenes::SceneError;

/// Apply vehicle restrictions to all connections (ucid=0 = global default).
///
/// - Standard vehicles are restricted via `Plc`.
/// - Mods are restricted via `Mal`. An empty mod list clears all mod restrictions.
///
/// Semantics of `vehicles`:
/// - Empty slice → no restrictions at all: `Plc` with all cars enabled, empty `Mal`.
/// - Non-empty slice → whitelist: only the listed vehicles are allowed.
///   Standard cars not in the list are blocked via `Plc`; mods not in the list are blocked via `Mal`.
///   If the slice contains only mods (no standard cars), `Plc` sends `bits=0` - no standard cars allowed.
pub async fn apply(insim: &InsimTask, vehicles: &[Vehicle]) -> Result<(), SceneError> {
    let mut mal = Mal::default();

    let cars = if vehicles.is_empty() {
        // No restriction - allow all standard cars.
        PlcAllowedCarsSet::all()
    } else {
        // Whitelist - only the specified standard cars.
        let mut cars = PlcAllowedCarsSet::default();
        for v in vehicles {
            match v {
                Vehicle::Mod(_) => {
                    let _ = mal.insert(*v);
                },
                _ => {
                    let _ = cars.insert(*v);
                },
            }
        }
        cars
    };

    insim
        .send(Plc {
            cars,
            ucid: ConnectionId::ALL,
            ..Plc::default()
        })
        .await
        .map_err(|cause| SceneError::Custom {
            scene: "vehicle_restrictions::plc",
            cause: Box::new(cause),
        })?;

    insim.send(mal).await.map_err(|cause| SceneError::Custom {
        scene: "vehicle_restrictions::mal",
        cause: Box::new(cause),
    })?;

    Ok(())
}
