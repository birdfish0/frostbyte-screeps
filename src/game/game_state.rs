//! Contains [GameState] and its implementation.

use std::{collections::VecDeque, time::Instant};

use anyhow::{Error, anyhow, bail, ensure};
use log::{debug, error, info};
use rand::SeedableRng;

use crate::{game::{*, room::*}, utils::{hash::SeaHashBuilder, event_queue, recursive_avg::RecursiveAvg}};

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
    pub creeps: HashMap<u64, Creep, SeaHashBuilder>,
    pub structures: HashMap<u64, Structure, SeaHashBuilder>,
    pub construction_sites: HashMap<u64, ConstructionSite, SeaHashBuilder>,
    pub ruins: HashMap<u64, Ruin, SeaHashBuilder>,
    pub tombstones: HashMap<u64, Tombstone, SeaHashBuilder>,
    pub dropped_resources: HashMap<u64, DroppedResource, SeaHashBuilder>,
    pub energy_sources: HashMap<u64, EnergySource, SeaHashBuilder>,
    pub mineral_sources: HashMap<u64, MineralSource, SeaHashBuilder>,
    pub commodity_sources: HashMap<u64, CommoditySource, SeaHashBuilder>,
    pub random: SmallRng,
}

#[derive(PartialEq, Eq, Clone, Debug)]
/// A small message sent from the render thread to the server thread.
/// Used to update properties pertaining to the process that calls [GameState::tick] or request a full game state update.
/// 
/// Note that full game state overrides are sent through their own channel, to keep these messages small in size.
pub enum RenderToServerThreadMsg {
    Quit,
    Pause(bool),
    ForceTickDelay(u32),
    LoadMapFile(String),
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

        let mut rng = SmallRng::seed_from_u64(4);

        Self{
            paused: false,
            tick: 0,
            force_tick_delay: 0,
            tick_durations: RecursiveAvg::new(vec![256, 32, 32], true),
            last_tick_time: Instant::now(),
            intents: VecDeque::new(),
            rooms: rooms,
            creeps: HashMap::with_hasher(SeaHashBuilder::new(&mut rng)),
            structures: HashMap::with_hasher(SeaHashBuilder::new(&mut rng)),
            construction_sites: HashMap::with_hasher(SeaHashBuilder::new(&mut rng)),
            ruins: HashMap::with_hasher(SeaHashBuilder::new(&mut rng)),
            tombstones: HashMap::with_hasher(SeaHashBuilder::new(&mut rng)),
            dropped_resources: HashMap::with_hasher(SeaHashBuilder::new(&mut rng)),
            energy_sources: HashMap::with_hasher(SeaHashBuilder::new(&mut rng)),
            mineral_sources: HashMap::with_hasher(SeaHashBuilder::new(&mut rng)),
            commodity_sources: HashMap::with_hasher(SeaHashBuilder::new(&mut rng)),
            random: rng,
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

    /// Modify [Room] objects inside [Self::rooms] to match the contents of a JSON file,
    /// in the format found on the ScreepsPlus map download page: <https://maps.screepspl.us/>.
    /// See assets/mmo-shard3.json for an example.
    /// 
    /// ## Arguments
    /// 
    /// - `data`: String containing the JSON object to be parsed.
    pub fn load_rooms_from_json(&mut self, data: &str) -> Result<(), Error> {
        info!("Loading map data from JSON");

        let jsonroot;
        match serde_json::from_str::<serde_json::Value>(&data) {
            Err(err) => {
                bail!("Malformed JSON: {}", err);
            },
            Ok(v) => jsonroot=v,
        }
        let rooms;
        match jsonroot["rooms"].as_array() {
            None => {
                bail!("Malformed room data, expected 'rooms' key at root level.");
            },
            Some(v) => rooms = v
        }
        for room in rooms {
            let room_name;
            match room["room"].as_str() {
                None => {
                    bail!("Malformed room data, expected 'room' key within room data to contain string.");
                }, 
                Some(v) => room_name = v,
            }
            let terrain; match room["terrain"].as_str() {
                None => {
                    bail!("Malformed terrain data, expected 'terrain' key within room data to contain string.");
                }, 
                Some(v) => terrain = v
            };

            let res = Room::pos_from_name(room_name);
            ensure!(res.is_ok(), "Failed to parse room name {}: {}", room_name, res.err().unwrap()); // Safe unwrap
            let (posx, posy) = res.unwrap(); // Safe unwrap
            let posx = posx as usize;
            let posy = posy as usize;

            ensure!(posy >= self.rooms.len(), "Y coordinate of room {} exceeded limits.", room_name);
            ensure!(posx < self.rooms[posy].len(), "X coordinate of room {} exceeded limits.", room_name);

            self.rooms[posy][posx].terrain_from_string(terrain);
        }
        Ok(())
    }
}