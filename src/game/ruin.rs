use super::*;

#[derive(Clone, Debug, Default)]
pub struct Ruin {
    id: u64,
    pos: RoomPos,
    store: Store,
    decay_timer: u128,
    death_time: u128,
    structure: Structure,
}