use super::*;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Creep {
    pub id: u64,
    pub name: String,

    pub pos: RoomPos,

    pub hits: u32,
    pub hits_max: u32,
    pub parts: [CreepPart; 50],
    pub num_parts: u8,

    pub fatigue: u32,
    pub spawning: bool,
}

impl Default for Creep {
    fn default() -> Self {
        Self {
            id: 0,
            name: "".to_owned(),
            pos: RoomPos::default(),
            hits: 0,
            hits_max: 0,
            parts: [CreepPart::default(); 50],
            num_parts: 0,
            fatigue: 0,
            spawning: false,
        }
    }
}