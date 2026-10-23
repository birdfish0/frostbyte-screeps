//! Contains the [Room] struct and its implementation, as well as functions related to rooms.

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Room {
    pub posx: u8,
    pub posy: u8,
    pub creeps: Vec<u64>,              // IDs
    pub terrain: [u128; 50], // (rows) bitfield of terrain, 2 bits per tile, 0=plain, 1=wall, 2=swamp
    pub structures: Vec<u64>, // IDs
    pub energy_sources: Vec<u64>, // IDs
    pub mineral_source: Option<u64>, // IDs
    pub commodity_source: Option<u64>, // IDs
    pub construction_sites: Vec<u64>, // IDs
}

impl Default for Room {
    fn default() -> Self {
        Self {
            posx: 0,
            posy: 0,
            creeps: Vec::new(),
            terrain: [0; 50],
            structures: Vec::new(),
            energy_sources: Vec::new(),
            mineral_source: None,
            commodity_source: None,
            construction_sites: Vec::new(),
        }
    }
}
