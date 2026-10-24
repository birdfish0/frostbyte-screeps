use super::*;

#[derive(PartialEq, Eq, Copy, Clone, Debug, Default)]
pub struct CommoditySource {
    pub id: u64,
    pub commodity_type: ResourceType,
    pub pos: RoomPos,
    pub amount: u32,
    pub refresh_timer: u128,
    pub density: MineralDensity,
}