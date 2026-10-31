//! Contains [GameState] and its implementation.

use std::{collections::VecDeque, time::Instant};

use anyhow::{Context, Error, bail, ensure};
use log::{debug, error, warn};
use rand::SeedableRng;

use crate::{
    game::*,
    utils::{event_queue::QueuedEvent, hash::SeaHashBuilder, recursive_avg::RecursiveAvg},
};

#[derive(Debug, Clone)]
/// An object that represents the current state of the game.
///
/// It contains the actual data for most objects that are referenced by ID elsewhere.
pub struct GameState {
    pub prev_paused: bool,
    pub paused: bool,
    pub tick: u128,
    pub force_tick_delay: u32,
    pub idgen: u64,
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
    pub is_server: bool,
}

impl GameState {
    /// Creates a [GameState] with default parameters.
    pub fn new() -> Self {
        let mut rooms: Vec<Vec<Room>> = (0..MAP_SIZE_Y)
            .map(|_| Vec::with_capacity(MAP_SIZE_X))
            .collect::<Vec<_>>();
        for x in 0..MAP_SIZE_X {
            for y in 0..MAP_SIZE_Y {
                rooms.get_mut(y).unwrap().push(Room {
                    posx: x as u8,
                    posy: y as u8,
                    ..Default::default()
                });
            }
        }

        let mut rng = SmallRng::seed_from_u64(4);

        Self {
            prev_paused: true,
            paused: true,
            tick: 0,
            force_tick_delay: 0,
            idgen: 1,
            tick_durations: RecursiveAvg::new(vec![64, 8, 8], true).unwrap_or_else(|err| {
                error!("Failed to create new RecursiveAvg: {}", err);
                Default::default()
            }),
            last_tick_time: Instant::now(),
            intents: VecDeque::new(),
            rooms,
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
            is_server: false,
        }
    }

    /// Run a single tick of the game. `event_queue` is provided separately here (instead of being stored in the GameState directly) because
    /// some events need to modify the [GameState], which is difficult to do when the event queue is contained within the GameState object.
    pub fn tick(&mut self, event_queue: &mut EventQueue) {
        if !self.paused {
            let tick_duration = self.last_tick_time.elapsed();
            self.last_tick_time = Instant::now();
            if !self.prev_paused {
                self.tick_durations.add(tick_duration.as_nanos());
            }
            self.tick += 1;

            while let Some(intent) = self.intents.pop_front() {
                self.process_intent(intent);
            }

            event_queue.tick(self.tick, self);
        }
        self.prev_paused = self.paused;
    }

    /// Process an [Intent] and change the game state accordingly.
    ///
    /// Note that some intents may return silently if they fail at this stage,
    /// such as [Move][Intent::Move], if the target position becomes blocked during a move.
    pub fn process_intent(&mut self, intent: Intent) {
        match intent {
            Intent::Nothing => (),
            Intent::Move(creepid, dir) => {
                todo!("{} {:?}", creepid, dir);
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
    pub fn load_rooms_from_json(
        &mut self,
        data: &str,
        event_queue: &mut EventQueue,
    ) -> Result<(), Error> {
        debug!("Loading map data from JSON");

        let jsonroot = match serde_json::from_str::<serde_json::Value>(data) {
            Err(err) => {
                bail!("Malformed JSON: {}", err);
            }
            Ok(v) => v,
        };
        let rooms = match jsonroot["rooms"].as_array() {
            None => {
                bail!("Malformed room data, expected 'rooms' key at root level.");
            }
            Some(v) => v,
        };
        for room in rooms {
            let room_name = match room["room"].as_str() {
                None => {
                    bail!(
                        "Malformed room data, expected 'room' key within room data to contain string."
                    );
                }
                Some(v) => v,
            };
            let terrain = match room["terrain"].as_str() {
                None => {
                    bail!(
                        "Malformed terrain data, expected 'terrain' key within room data to contain string."
                    );
                }
                Some(v) => v,
            };

            let res = Room::pos_from_name(room_name);
            ensure!(
                res.is_ok(),
                "Failed to parse room name {}: {}",
                room_name,
                res.err().unwrap()
            ); // Safe unwrap
            let (posx, posy) = res.unwrap(); // Safe unwrap
            let posx = posx as usize;
            let posy = posy as usize;

            ensure!(
                posy < self.rooms.len(),
                "Y coordinate of room {} exceeded limits.",
                room_name
            );
            ensure!(
                posx < self.rooms[posy].len(),
                "X coordinate of room {} exceeded limits.",
                room_name
            );

            self.rooms[posy][posx].terrain_from_string(terrain)?;

            let objs = match room["objects"].as_array() {
                None => {
                    bail!(
                        "Malformed room data on room {}, expected 'objects' key within room data to contain array.",
                        room_name
                    );
                }
                Some(v) => v,
            };

            for obj in objs {
                match match obj["type"].as_str() {
                    None => {
                        bail!(
                            "Malformed room data on room {}, expected index of 'objects' array to contain key 'type' with a str value.",
                            room_name
                        );
                    }
                    Some(v) => v,
                } {
                    "extractor" => {
                        // TODO
                    }
                    "keeperLair" => {
                        // TODO
                    }
                    "deposit" => {
                        // TODO
                    }
                    "source" => {
                        let x = obj["x"].as_u64().with_context(|| format!("Malformed room data on room {}, expected 'source' in 'objects' array to contain key 'x' with a u64 value.", room_name))?;
                        let y = obj["y"].as_u64().with_context(|| format!("Malformed room data on room {}, expected 'source' in 'objects' array to contain key 'y' with a u64 value.", room_name))?;
                        self.rooms[posy][posx].energy_sources.insert(self.idgen);
                        self.energy_sources.insert(
                            self.idgen,
                            EnergySource {
                                amount: UNCLAIMED_ROOM_SRC_AMOUNT,
                                id: self.idgen,
                                max_amount: UNCLAIMED_ROOM_SRC_AMOUNT,
                                pos: RoomPos {
                                    room_x: posx as u8,
                                    room_y: posy as u8,
                                    pos_x: x as u8,
                                    pos_y: y as u8,
                                },
                                refresh_timer: 0,
                            },
                        );
                        event_queue.enqueue(QueuedEvent {
                            callback: Box::new(EnergySourceRefreshEvent { id: self.idgen }),
                            tick: 0,
                        });
                        self.idgen += 1;
                    }
                    "controller" => {
                        let x = obj["x"].as_u64().with_context(|| format!("Malformed room data on room {}, expected 'controller' in 'objects' array to contain key 'x' with a u64 value.", room_name))?;
                        let y = obj["y"].as_u64().with_context(|| format!("Malformed room data on room {}, expected 'controller' in 'objects' array to contain key 'y' with a u64 value.", room_name))?;
                        self.rooms[posy][posx].structures.insert(self.idgen);
                        self.structures.insert(
                            self.idgen,
                            Structure {
                                id: self.idgen,
                                structure_type: StructureType::Controller,
                                pos: RoomPos {
                                    room_x: posx as u8,
                                    room_y: posy as u8,
                                    pos_x: x as u8,
                                    pos_y: y as u8,
                                },
                                hits: 0,
                                hits_max: 0,
                            },
                        );
                        self.idgen += 1;
                    }
                    "mineral" => {
                        let x = obj["x"].as_u64().with_context(|| format!("Malformed room data on room {}, expected 'mineral' in 'objects' array to contain key 'x' with a u64 value.", room_name))?;
                        let y = obj["y"].as_u64().with_context(|| format!("Malformed room data on room {}, expected 'mineral' in 'objects' array to contain key 'y' with a u64 value.", room_name))?;
                        let amount = obj["mineralAmount"].as_u64().with_context(|| format!("Malformed room data on room {}, expected 'mineral' in 'objects' array to contain key 'mineralAmount' with a u64 value.", room_name))?;
                        let mtype = obj["mineralType"].as_str().with_context(|| format!("Malformed room data on room {}, expected 'mineral' in 'objects' array to contain key 'mineralType' with a str value.", room_name))?;
                        let mtype = match str2type(mtype) {
                            Some(x) => x,
                            _ => bail!(
                                "Malformed room data on room {}, unknown mineral type {}.",
                                room_name,
                                mtype
                            ),
                        };
                        let density = obj["density"].as_u64().with_context(|| format!("Malformed room data on room {}, expected 'mineral' in 'objects' array to contain key 'density' with a u64 value.", room_name))?;
                        let density = match density {
                            1 => MineralDensity::Low,
                            2 => MineralDensity::Medium,
                            3 => MineralDensity::High,
                            4 => MineralDensity::Ultra,
                            _ => bail!(
                                "Malformed room data on room {}, unknown density value {}.",
                                room_name,
                                density
                            ),
                        };
                        self.rooms[posy][posx].mineral_source = Some(self.idgen);
                        self.mineral_sources.insert(
                            self.idgen,
                            MineralSource {
                                id: self.idgen,
                                mineral_type: mtype,
                                pos: RoomPos {
                                    room_x: posx as u8,
                                    room_y: posy as u8,
                                    pos_x: x as u8,
                                    pos_y: y as u8,
                                },
                                amount: amount as u32,
                                refresh_timer: 0,
                                density,
                            },
                        );
                        event_queue.enqueue(QueuedEvent {
                            callback: Box::new(MineralRefreshEvent { id: self.idgen }),
                            tick: 0,
                        });
                        self.idgen += 1;
                    }
                    some => {
                        warn!(
                            "Skipping unknown object of type '{}' in room {}'s data.",
                            some, room_name
                        );
                    }
                };
            }
        }
        Ok(())
    }
}
