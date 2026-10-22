use std::{collections::HashMap, sync::Mutex};

use rand::{RngExt, rngs::SmallRng};

use crate::{
    consts::*,
    utils::{self, event_queue::EventQueue},
};
use enums::*;
use mineral::*;

pub mod enums;
pub mod mineral;

pub struct GameState {
    pub paused: bool,
    pub tick: u128,
    pub intents: Vec<Intent>,
    pub rooms: Mutex<[[Room; 122]; 122]>,
    pub creeps: Mutex<HashMap<u64, Creep>>,
    pub energy_sources: Mutex<HashMap<u64, EnergySource>>,
    pub mineral_sources: Mutex<HashMap<u64, MineralSource>>,
    pub commodity_sources: Mutex<HashMap<u64, CommoditySource>>,
    pub structures: Mutex<HashMap<u64, Structure>>,
    pub random: SmallRng,
}

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

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct RoomPos {
    pub room_x: u8,
    pub room_y: u8,
    pub pos_x: u8,
    pub pos_y: u8,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct EnergySource {
    pub id: u64,
    pub pos: RoomPos,
    pub amount: u32,
    pub refresh_timer: u128,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct CommoditySource {
    pub id: u64,
    pub commodity_type: ResourceType,
    pub pos: RoomPos,
    pub amount: u32,
    pub refresh_timer: u128,
    pub density: MineralDensity,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum MineralDensity {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct Structure {
    pub id: u64,
    pub structure_type: StructureType,
    pub pos: RoomPos,
    pub hits: u32,
    pub hits_max: u32,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct ConstructionSite {
    pub id: u64,
    pub structure_type: StructureType,
    pub pos: RoomPos,
    pub energy: u64,
}

pub struct Store {
    pub contents: HashMap<ResourceType, u32>,
    pub single_type: Option<ResourceType>,
    pub capacity: u32,
    pub used_capacity: u32,
}
