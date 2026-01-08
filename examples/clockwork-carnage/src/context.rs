pub trait HasInsim {
    fn insim(&self) -> insim::builder::SpawnedHandle;
}

pub trait HasPresence {
    fn presence(&self) -> kitcar::presence::PresenceHandle;
}

pub trait HasGame {
    fn game(&self) -> kitcar::game::GameHandle;
}

pub trait HasChat {
    fn chat(&self) -> crate::chat::ChatHandle;
}
