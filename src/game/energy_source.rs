use super::*;

#[derive(PartialEq, Eq, Copy, Clone, Debug, Default)]
pub struct EnergySource {
    pub id: u64,
    pub pos: RoomPos,
    pub amount: u32,
    pub refresh_timer: u128,
}
