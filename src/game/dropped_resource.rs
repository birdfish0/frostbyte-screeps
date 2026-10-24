use super::*;

#[derive(Copy, Clone, Debug, Default)]
pub struct DroppedResource {
    id: u64,
    pos: RoomPos,
    resource_type: ResourceType,
    amount: u32,
    last_amount_at_decay_change: u32,
    decay_change_time: u128,
    decay_rate: u32,
}
