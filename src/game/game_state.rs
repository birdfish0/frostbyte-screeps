//! Contains [GameState] and its implementation.

use rand::SeedableRng;

use crate::{game::*, utils::event_queue};

#[derive(Debug)]
pub struct GameState {
    pub paused: bool,
    pub tick: u128,
    pub intents: Vec<Intent>,
    pub rooms: Vec<Vec<Room>>,
    pub creeps: HashMap<u64, Creep>,
    pub structures: HashMap<u64, Structure>,
    pub energy_sources: HashMap<u64, EnergySource>,
    pub mineral_sources: HashMap<u64, MineralSource>,
    pub commodity_sources: HashMap<u64, CommoditySource>,
    pub random: SmallRng,
    pub event_queue: EventQueue,
}

impl GameState {
    pub fn new() -> Self {
        let mut rooms: Vec<Vec<Room>> = vec![Vec::with_capacity(MAP_SIZE_X); MAP_SIZE_Y];
        for x in 0..MAP_SIZE_X {
            for y in 0..MAP_SIZE_Y {
                rooms.get_mut(y).unwrap().push(Room {
                    posx: x as u8,
                    posy: y as u8,
                    terrain: [0; 50],
                    commodity_source: None,
                    construction_sites: Vec::new(),
                    creeps: Vec::new(),
                    energy_sources: Vec::new(),
                    mineral_source: None,
                    structures: Vec::new(),
                });
            }
        }
        Self{
            paused: false,
            tick: 0,
            intents: Vec::new(),
            rooms: rooms,
            creeps: HashMap::new(),
            structures: HashMap::new(),
            energy_sources: HashMap::new(),
            mineral_sources: HashMap::new(),
            commodity_sources: HashMap::new(),
            random: SmallRng::seed_from_u64(4),
            event_queue: EventQueue::new(),
        }
    }

    pub fn tick(&mut self, event_queue: &mut EventQueue) {
        event_queue.tick(self.tick, self);
    }

    pub fn process_intent(&mut self, intent: Intent) {
        match intent {
            Intent::Move(creepid, dir) => {
                todo!();
            }
        }
    }
}