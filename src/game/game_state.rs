//! Contains [GameState] and its implementation.

use std::{collections::VecDeque, time::Instant};

use rand::SeedableRng;

use crate::{game::*, utils::{event_queue, recursive_avg::RecursiveAvg}};

#[derive(Debug, Clone)]
/// An object that represents the current state of the game.
/// 
/// It contains the actual data for most objects that are referenced by ID elsewhere.
pub struct GameState {
    pub paused: bool,
    pub tick: u128,
    pub force_tick_delay: u32,
    pub tick_durations: RecursiveAvg,
    pub last_tick_time: Instant,
    pub intents: VecDeque<Intent>,
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
/// A small message sent from the render thread to the server thread.
/// Used to update properties pertaining to the process that calls [GameState::tick] or request a full game state update.
/// 
/// Note that full game state overrides are sent through their own channel, to keep these messages small in size.
pub enum RenderToServerThreadMsg {
    Quit,
    Pause(bool),
    ForceTickDelay(u32),
    RequestFullState,
}

impl GameState {
    /// Creates a [GameState] with default parameters.
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
            intents: VecDeque::new(),
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

    /// Run a single tick of the game. `event_queue` is provided separately here (instead of being stored in the GameState directly) because
    /// some events need to modify the [GameState], which is difficult to do when the event queue is contained within the GameState object.
    pub fn tick(&mut self, event_queue: &mut EventQueue) {
        let tick_duration = self.last_tick_time.elapsed();
        self.tick_durations.add(tick_duration.as_nanos());
        self.tick += 1;

        while let Some(intent) = self.intents.pop_front() {
            self.process_intent(intent);
        }

        event_queue.tick(self.tick, self);
        self.last_tick_time = Instant::now();
    }

    /// Process an [Intent] and change the game state accordingly.
    /// 
    /// Note that some intents may return silently if they fail at this stage,
    /// such as [Move][Intent::Move], if the target position becomes blocked during a move.
    pub fn process_intent(&mut self, intent: Intent) {
        match intent {
            Intent::Nothing => (),
            Intent::Move(creepid, dir) => {
                todo!();
            }
        }
    }
}