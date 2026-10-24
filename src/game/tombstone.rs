use super::*;

#[derive(Clone, Debug, Default)]
pub struct Tombstone {
    id: u64,
    pos: RoomPos,
    store: Store,
    decay_timer: u128,
    death_time: u128,
    creep: Creep,
}