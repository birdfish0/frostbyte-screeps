//! Contains [GameState] and its implementation.

use crate::game::*;

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
    pub event_queue: EventQueue,
}