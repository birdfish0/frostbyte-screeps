//! Includes things directly relevant to the game content, such as the [GameState], or [Creep].

use std::collections::HashMap;

use rand::{RngExt, rngs::SmallRng};

use crate::{
    consts::*,
    utils::{self, event_queue::EventQueue},
};
use enums::*;
use game_state::*;
use mineral::*;

pub mod enums;
pub mod game_state;
pub mod mineral;
pub mod room;

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

#[derive(Clone, Debug, Default)]
pub struct Ruin {
    id: u64,
    pos: RoomPos,
    store: Store,
    decay_timer: u128,
    death_time: u128,
    structure: Structure,
}

#[derive(Clone, Debug, Default)]
pub struct Tombstone {
    id: u64,
    pos: RoomPos,
    store: Store,
    decay_timer: u128,
    death_time: u128,
    creep: Creep,
}

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

#[derive(PartialEq, Eq, Copy, Clone, Debug, Default)]
pub struct RoomPos {
    pub room_x: u8,
    pub room_y: u8,
    pub pos_x: u8,
    pub pos_y: u8,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Default)]
pub struct EnergySource {
    pub id: u64,
    pub pos: RoomPos,
    pub amount: u32,
    pub refresh_timer: u128,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Default)]
pub struct CommoditySource {
    pub id: u64,
    pub commodity_type: ResourceType,
    pub pos: RoomPos,
    pub amount: u32,
    pub refresh_timer: u128,
    pub density: MineralDensity,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Default)]
pub struct Structure {
    pub id: u64,
    pub structure_type: StructureType,
    pub pos: RoomPos,
    pub hits: u32,
    pub hits_max: u32,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Default)]
pub struct ConstructionSite {
    pub id: u64,
    pub structure_type: StructureType,
    pub pos: RoomPos,
    pub energy: u64,
}

#[derive(Clone, Debug, Default)]
pub struct Store {
    pub contents: HashMap<ResourceType, u32>,
    pub single_type: Option<ResourceType>,
    pub capacity: u32,
    pub used_capacity: u32,
}
