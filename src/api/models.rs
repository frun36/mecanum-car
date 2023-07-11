use crate::drive::Motion;
use crate::drive::Speed;

#[derive(serde::Deserialize)]
pub struct DriveParams {
    pub direction: Motion,
    pub speed: Speed,
}
