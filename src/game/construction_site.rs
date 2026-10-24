use super::*;

#[derive(PartialEq, Eq, Copy, Clone, Debug, Default)]
pub struct ConstructionSite {
    pub id: u64,
    pub structure_type: StructureType,
    pub pos: RoomPos,
    pub energy: u64,
}
