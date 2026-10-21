use std::{collections::HashMap, sync::Mutex};

use rand::{RngExt, rngs::SmallRng};

use crate::{consts::*, util};

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

pub struct RoomPos {
    pub room_x: u8,
    pub room_y: u8,
    pub pos_x: u8,
    pub pos_y: u8,
}

pub struct EnergySource {
    pub id: u64,
    pub pos: RoomPos,
    pub amount: u32,
    pub refresh_timer: u128,
}

pub struct MineralSource {
    pub id: u64,
    pub mineral_type: ResourceType,
    pub pos: RoomPos,
    pub amount: u32,
    pub refresh_timer: u128,
    pub density: MineralDensity,
}

impl MineralSource {
    pub fn refresh(&mut self, game_state: &mut GameState) {
        if self.density == MineralDensity::Low
            || self.density == MineralDensity::Ultra
            || game_state.random.random_ratio(MINERAL_DENSITY_CHANGE_CHANCE_PERCENT, 100)
        {
            self.density = match self.density {
                MineralDensity::Low => util::random_choice(
                    &mut game_state.random,
                    &[
                        MineralDensity::Medium,
                        MineralDensity::High,
                        MineralDensity::Ultra,
                    ],
                    &[
                        MINERAL_DENSITY_MEDIUM_CHANCE_PER1000,
                        MINERAL_DENSITY_HIGH_CHANCE_PER1000,
                        MINERAL_DENSITY_ULTRA_CHANCE_PER1000,
                    ],
                ),
                MineralDensity::Medium => util::random_choice(
                    &mut game_state.random,
                    &[
                        MineralDensity::Low,
                        MineralDensity::High,
                        MineralDensity::Ultra,
                    ],
                    &[
                        MINERAL_DENSITY_LOW_CHANCE_PER1000,
                        MINERAL_DENSITY_HIGH_CHANCE_PER1000,
                        MINERAL_DENSITY_ULTRA_CHANCE_PER1000,
                    ],
                ),
                MineralDensity::High => util::random_choice(
                    &mut game_state.random,
                    &[
                        MineralDensity::Low,
                        MineralDensity::Medium,
                        MineralDensity::Ultra,
                    ],
                    &[
                        MINERAL_DENSITY_LOW_CHANCE_PER1000,
                        MINERAL_DENSITY_MEDIUM_CHANCE_PER1000,
                        MINERAL_DENSITY_ULTRA_CHANCE_PER1000,
                    ],
                ),
                MineralDensity::Ultra => util::random_choice(
                    &mut game_state.random,
                    &[
                        MineralDensity::Low,
                        MineralDensity::Medium,
                        MineralDensity::High,
                    ],
                    &[
                        MINERAL_DENSITY_LOW_CHANCE_PER1000,
                        MINERAL_DENSITY_MEDIUM_CHANCE_PER1000,
                        MINERAL_DENSITY_HIGH_CHANCE_PER1000,
                    ],
                ),
            }
        }
        self.amount = match self.density {
            MineralDensity::Low => MINERAL_DENSITY_LOW_VALUE,
            MineralDensity::Medium => MINERAL_DENSITY_MEDIUM_VALUE,
            MineralDensity::High => MINERAL_DENSITY_HIGH_VALUE,
            MineralDensity::Ultra => MINERAL_DENSITY_ULTRA_VALUE,
        };
        self.refresh_timer = game_state.tick + MINERAL_REGEN_TIME as u128;
    }
}

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

pub struct Structure {
    pub id: u64,
    pub structure_type: StructureType,
    pub pos: RoomPos,
    pub hits: u32,
    pub hits_max: u32,
}

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

pub enum StructureType {
    Spawn,
    Extension,
    Road,
    Wall,
    Rampart,
    KeeperLair,
    Portal,
    Controller,
    Link,
    Storage,
    Tower,
    Observer,
    PowerBank,
    PowerSpawn,
    Extractor,
    Lab,
    Terminal,
    Container,
    Nuker,
}

pub enum ResourceType {
    Energy,
    Power,
    Hydrogen,
    Oxygen,
    Utrium,
    Lemergium,
    Keanium,
    Zynthium,
    Catalyst,
    Ghodium,

    Silicon,
    Metal,
    Biomass,
    Mist,

    Hydroxide,
    ZynthiumKeanite,
    UtriumLemergite,

    UtriumHydride,
    UtriumOxide,
    KeaniumHydride,
    KeaniumOxide,
    LemergiumHydride,
    LemergiumOxide,
    ZynthiumHydride,
    ZynthiumOxide,
    GhodiumHydride,
    GhodiumOxide,

    UtriumAcid,
    UtriumAlkalide,
    KeaniumAcid,
    KeaniumAlkalide,
    LemergiumAcid,
    LemergiumAlkalide,
    ZynthiumAcid,
    ZynthiumAlkalide,
    GhodiumAcid,
    GhodiumAlkalide,

    CatalyzedUtriumAcid,
    CatalyzedUtriumAlkalide,
    CatalyzedKeaniumAcid,
    CatalyzedKeaniumAlkalide,
    CatalyzedLemergiumAcid,
    CatalyzedLemergiumAlkalide,
    CatalyzedZynthiumAcid,
    CatalyzedZynthiumAlkalide,
    CatalyzedGhodiumAcid,
    CatalyzedGhodiumAlkalide,

    Ops,

    Utriumbar,
    Lemergiumbar,
    Zynthiumbar,
    Keaniumbar,
    Ghodiummelt,
    Oxidant,
    Reductant,
    Purifier,
    Battery,

    Composite,
    Crystal,
    Liquid,

    Wire,
    Switch,
    Transistor,
    Microchip,
    Circuit,
    Device,

    Cell,
    Phlegm,
    Tissue,
    Muscle,
    Organoid,
    Organism,

    Alloy,
    Tube,
    Fixtures,
    Frame,
    Hydraulics,
    Machine,

    Condensate,
    Concentrate,
    Extract,
    Spirit,
    Emanation,
    Essence,
}

pub enum CreepPart {
    None,
    Move,
    Move1,
    Move2,
    Move3,
    Work,
    Work1,
    Work2,
    Work3,
    Carry,
    Carry1,
    Carry2,
    Carry3,
    Attack,
    Attack1,
    Attack2,
    Attack3,
    RangedAttack,
    RangedAttack1,
    RangedAttack2,
    RangedAttack3,
    Heal,
    Heal1,
    Heal2,
    Heal3,
    Claim,
    Claim1,
    Claim2,
    Claim3,
    Tough,
    Tough1,
    Tough2,
    Tough3,
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

pub enum Intent {
    Move(u64, Direction), // ID, direction
}
