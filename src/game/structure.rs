use super::*;

#[derive(PartialEq, Eq, Copy, Clone, Debug, Default)]
pub struct Structure {
    pub id: u64,
    pub structure_type: StructureType,
    pub pos: RoomPos,
    pub hits: u32,
    pub hits_max: u32,
}