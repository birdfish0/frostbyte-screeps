//! Contains [GameState] and its implementation.

use std::time::Instant;

use rand::SeedableRng;

use crate::{game::*, utils::{event_queue, recursive_avg::RecursiveAvg}};

#[derive(Debug, Clone)]
pub struct GameState {
    pub paused: bool,
    pub tick: u128,
    pub force_tick_delay: u32,
    pub tick_durations: RecursiveAvg,
    pub last_tick_time: Instant,
    pub intents: Vec<Intent>,
    pub rooms: Vec<Vec<Room>>,
    pub creeps: HashMap<u64, Creep>,
    pub structures: HashMap<u64, Structure>,
    pub construction_sites: HashMap<u64, ConstructionSite>,
    pub ruins: HashMap<u64, Ruin>,
    pub tombstones: HashMap<u64, Tombstone>,
    pub dropped_resources: HashMap<u64, DroppedResource>,
    pub energy_sources: HashMap<u64, EnergySource>,
    pub mineral_sources: HashMap<u64, MineralSource>,
    pub commodity_sources: HashMap<u64, CommoditySource>,
    pub random: SmallRng,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum RenderToServerThreadMsg {
    Quit,
    Pause(bool),
    ForceTickDelay(u32),
    RequestFullState,
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
            force_tick_delay: 0,
            tick_durations: RecursiveAvg::new(vec![200, 10, 100], true),
            last_tick_time: Instant::now(),
            intents: Vec::new(),
            rooms: rooms,
            creeps: HashMap::new(),
            structures: HashMap::new(),
            construction_sites: HashMap::new(),
            ruins: HashMap::new(),
            tombstones: HashMap::new(),
            dropped_resources: HashMap::new(),
            energy_sources: HashMap::new(),
            mineral_sources: HashMap::new(),
            commodity_sources: HashMap::new(),
            random: SmallRng::seed_from_u64(4),
        }
    }

    pub fn tick(&mut self, event_queue: &mut EventQueue) {
        let tick_duration = self.last_tick_time.elapsed();
        self.tick_durations.add(tick_duration.as_nanos());
        self.tick += 1;
        event_queue.tick(self.tick, self);
        self.last_tick_time = Instant::now();
    }

    pub fn process_intent(&mut self, intent: Intent) {
        match intent {
            Intent::Nothing => (),
            Intent::Move(creepid, dir) => {
                todo!();
            }
        }
    }
}