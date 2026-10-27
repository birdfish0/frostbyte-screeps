//! Contains the [Room] struct and its implementation, as well as functions related to rooms.

use std::{collections::HashSet, fmt::Display};

use anyhow::{Error, bail, ensure};
use log::error;
use rand::{Rng, rng};
use seahash::SeaHasher;

use crate::utils::{event_queue::QueuedEvent, hash::SeaHashBuilder, random::random_choice};

use super::*;

/// Contains data pertaining to a room on the map.
///
/// Does not contain the actual data of most of the objects within the room, only their ID.
#[derive(Clone, Debug)]
pub struct Room {
    /// X position of the room in the map, with 0 being the leftmost column.
    pub posx: u8,
    /// Y position of the room in the map, with 0 being the top row.
    pub posy: u8,
    /// IDs of [Creep]s in the room.
    pub creeps: HashSet<u64, SeaHashBuilder>,
    /// Terrain of the room, by rows. Each tile is represented by 2 bits,
    /// where `0b00` is plains, `0b01` is wall, `0b10` is swamp.
    ///
    /// This means that the first 100 bits will represent the first row of the terrain (top row),
    /// and bits 101-200 represent the second row, etc.
    pub terrain: [u8; 625],
    /// IDs of [Structure]s in the room.
    pub structures: HashSet<u64, SeaHashBuilder>,
    /// IDs of [EnergySource]s in the room.
    pub energy_sources: HashSet<u64, SeaHashBuilder>,
    /// ID of the [MineralSource] in the room, if there is one.
    pub mineral_source: Option<u64>,
    /// ID of the [CommoditySource] in the room, if there is one.
    pub commodity_source: Option<u64>,
    /// IDs of [ConstructionSite]s in the room.
    pub construction_sites: HashSet<u64, SeaHashBuilder>,
    /// IDs of [Ruin]s in the room.
    pub ruins: HashSet<u64, SeaHashBuilder>,
    /// IDs of [Tombstone]s in the room.
    pub tombstones: HashSet<u64, SeaHashBuilder>,
    /// IDs of [DroppedResource]s in the room.
    pub dropped_resources: HashSet<u64, SeaHashBuilder>,
}

impl Default for Room {
    fn default() -> Self {
        Self {
            posx: 0,
            posy: 0,
            creeps: HashSet::with_hasher(SeaHashBuilder::new(&mut rng())),
            terrain: [0; 625],
            structures: HashSet::with_hasher(SeaHashBuilder::new(&mut rng())),
            energy_sources: HashSet::with_hasher(SeaHashBuilder::new(&mut rng())),
            mineral_source: None,
            commodity_source: None,
            construction_sites: HashSet::with_hasher(SeaHashBuilder::new(&mut rng())),
            ruins: HashSet::with_hasher(SeaHashBuilder::new(&mut rng())),
            tombstones: HashSet::with_hasher(SeaHashBuilder::new(&mut rng())),
            dropped_resources: HashSet::with_hasher(SeaHashBuilder::new(&mut rng())),
        }
    }
}

/// A tile in the terrain of a room.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum TerrainTile {
    Plains,
    Wall,
    Swamp,
    #[default]
    /// Default value, should not appear normally in a room.
    Void,
}

impl From<u8> for TerrainTile {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Plains,
            1 => Self::Wall,
            2 => Self::Swamp,
            _ => Self::Void,
        }
    }
}

/// Minimum distance between two objects spread out using [Room::find_spread_position].
/// If a position of lesser distance is not possible, the function throws an error.
const ROOMGEN_MINIMUM_SPREAD_DIST: u8 = 14;

/// Amount that the CDTA of the returned point from [Room::find_spread_position] can deviate from the optimal value.
///
/// CDTA: Closest distance to anchor.
///
/// The optimal value is multiplied by this value to obtain the minimum allowed CDTA.
const ROOMGEN_SPREAD_DIST_TOLERANCE: f32 = 0.7;

impl Room {
    /// Same as [Room::name_from_pos], but cannot fail. Returns `<Unknown Room at (x, y)>` if `name_from_pos` fails.
    pub fn name_from_pos_checked<T>(posx: T, posy: T) -> String
    where
        T: Into<usize> + Display + Copy,
    {
        Self::name_from_pos(posx, posy).unwrap_or(format!("<Unknown Room at ({}, {})>", posx, posy))
    }

    /// Takes the position of a room in the world map and returns its name. i.e. (0, 0) gives "W60N60".
    pub fn name_from_pos<T>(posx: T, posy: T) -> Result<String, Error>
    where
        T: Into<usize> + Display + Copy,
    {
        let posx = posx.into();
        let posy = posy.into();
        ensure!(
            posx < MAP_SIZE_X,
            "Room X coordinate out of range: {}",
            posx
        );
        ensure!(
            posy < MAP_SIZE_Y,
            "Room Y coordinate out of range: {}",
            posy
        );
        let mut output = String::new();
        if posx < MAP_SIZE_X / 2 {
            output += "W";
            output += &(MAP_SIZE_X / 2 - 1 - posx).to_string();
        } else {
            output += "E";
            output += &(posx - MAP_SIZE_X / 2).to_string();
        }
        if posy < MAP_SIZE_Y / 2 {
            output += "N";
            output += &(MAP_SIZE_Y / 2 - 1 - posy).to_string();
        } else {
            output += "S";
            output += &(posy - MAP_SIZE_Y / 2).to_string();
        }
        Ok(output)
    }

    /// Generate a position within the room which is spread apart from the provided `anchor` positions.
    ///
    /// Uses logic from: <https://github.com/laverdet/xxscreeps/blob/main/packages/xxscreeps/scripts/room-gen.ts#L815>
    ///
    /// The `padmin` and `padmax` arguments determine the minimum and maximum X and Y coordinates of the returned position.
    ///
    /// ## Errors
    ///
    /// Errors if `padmin` or `padmax` exceed the boundaries of the room.
    ///
    /// Errors if no position at least [ROOMGEN_MINIMUM_SPREAD_DIST] away from all anchors is found.
    pub fn find_spread_position<T>(
        &self,
        padmin: u8,
        padmax: u8,
        anchors: &Vec<RoomPos>,
        rng: &mut T,
        avoid: Option<&Vec<RoomPos>>,
    ) -> Result<RoomPos, Error>
    where
        T: Rng,
    {
        let mut best: u8 = 0;
        let mut eligible = Vec::<(u8, u8)>::new();
        for x in padmin..padmax {
            for y in padmin..padmax {
                if let Some(avoid) = avoid {
                    if avoid.iter().any(|p| p.pos_x == x && p.pos_y == y) {
                        continue;
                    }
                }
                if self.get_look_terrain_area_at(x, y).iter().any(|row| {
                    row.iter()
                        .any(|t| *t == TerrainTile::Plains || *t == TerrainTile::Swamp)
                }) {
                    eligible.push((x, y));
                    best = best.max(
                        anchors
                            .iter()
                            .map(|anc| anc.get_range_to(x, y))
                            .max()
                            .unwrap_or_default(),
                    );
                }
            }
        }
        ensure!(
            best >= ROOMGEN_MINIMUM_SPREAD_DIST,
            "Could not find position sufficiently far ({} tiles) from anchors {:#?} in room {}.",
            ROOMGEN_MINIMUM_SPREAD_DIST,
            anchors,
            Self::name_from_pos_checked(self.posx, self.posy)
        );
        let threshold = ROOMGEN_MINIMUM_SPREAD_DIST
            .max((best as f32 * ROOMGEN_SPREAD_DIST_TOLERANCE).ceil() as u8);

        let eligible = eligible
            .iter()
            .filter(|(x, y)| anchors.iter().all(|a| a.get_range_to(*x, *y) >= threshold))
            .collect::<Vec<&(u8, u8)>>();

        let chosen = eligible[rng.random_range(0..eligible.len())];

        Ok(RoomPos {
            room_x: self.posx,
            room_y: self.posy,
            pos_x: chosen.0,
            pos_y: chosen.1,
        })
    }

    /// Populate the room's mineral source and energy sources from the given options.
    /// Returns the mineral source if one was requested, controller if requested, and a list of energy sources in the room.
    ///
    /// The mineral and energy source refresh events are automatically queued into the provided `event_queue`.
    pub fn generate<T>(
        &mut self,
        rng: &mut T,
        mineral: Option<ResourceType>,
        controller: bool,
        event_queue: &mut EventQueue,
        id: &mut u64,
    ) -> Result<
        (
            Option<MineralSource>,
            Option<Structure>,
            [Option<EnergySource>; 3],
        ),
        Error,
    >
    where
        T: Rng,
    {
        let mut src_positions = Vec::<RoomPos>::new();

        for _ in 0..rng.random_range(1..2) {
            src_positions.push(self.find_spread_position(3, 46, &src_positions, rng, None)?);
        }

        let mut out_srcs: [Option<EnergySource>; 3] = [None; 3];

        let mut i = 0;
        for pos in &src_positions {
            let src = EnergySource {
                id: *id,
                pos: pos.clone(),
                amount: UNCLAIMED_ROOM_SRC_AMOUNT,
                max_amount: UNCLAIMED_ROOM_SRC_AMOUNT,
                refresh_timer: 0,
            };
            out_srcs[i] = Some(src);
            *id += 1;
            i += 1;

            event_queue.enqueue(QueuedEvent {
                tick: ENERGY_SOURCE_REFRESH_TICKS,
                callback: Box::new(EnergySourceRefreshEvent { id: src.id }),
            });
        }

        let ctrler;
        let ctrler_position;
        if controller {
            ctrler_position =
                self.find_spread_position(5, 44, &vec![], rng, Some(&src_positions))?;
            ctrler = Some(Structure {
                hits: 0,
                hits_max: 0,
                id: *id,
                pos: ctrler_position,
                structure_type: StructureType::Controller,
            });
            *id += 1;
        } else {
            ctrler = None;
            ctrler_position = RoomPos::default();
        }

        let mineralsrc;
        if let Some(mineral) = mineral {
            let mut avoid = src_positions.clone();
            avoid.push(ctrler_position);
            let mineral_position;
            if src_positions.len() >= 3 {
                mineral_position =
                    self.find_spread_position(4, 45, &src_positions, rng, Some(&avoid))?;
            } else {
                mineral_position = self.find_spread_position(4, 45, &vec![], rng, Some(&avoid))?;
            }
            let src = MineralSource {
                amount: 0,
                density: random_choice(
                    rng,
                    &[
                        MineralDensity::Low,
                        MineralDensity::Medium,
                        MineralDensity::High,
                        MineralDensity::Ultra,
                    ],
                    &[
                        MINERAL_DENSITY_LOW_CHANCE_PER1000,
                        MINERAL_DENSITY_MEDIUM_CHANCE_PER1000,
                        MINERAL_DENSITY_HIGH_CHANCE_PER1000,
                        MINERAL_DENSITY_ULTRA_CHANCE_PER1000,
                    ],
                )
                .unwrap_or_else(|err| {
                    error!("Failed to generate new mineral density: {}", err);
                    Default::default()
                }),
                id: *id,
                mineral_type: mineral,
                pos: mineral_position,
                refresh_timer: 0,
            };
            *id += 1;
            event_queue.enqueue(QueuedEvent {
                callback: Box::new(MineralRefreshEvent { id: src.id }),
                tick: MINERAL_REGEN_TIME as u128,
            });
            mineralsrc = Some(src);
        } else {
            mineralsrc = None;
        }

        Ok((mineralsrc, ctrler, out_srcs))
    }

    /// Get the 9 terrain tiles around the given coordinates in the room.
    ///
    /// If a tile is out of range, [TerrainTile::Void] is returned in its place.
    pub fn get_look_terrain_area_at(&self, x: u8, y: u8) -> [[TerrainTile; 3]; 3] {
        [
            [
                self.get_terrain_at(x - 1, y - 1).unwrap_or_default(),
                self.get_terrain_at(x, y - 1).unwrap_or_default(),
                self.get_terrain_at(x + 1, y - 1).unwrap_or_default(),
            ],
            [
                self.get_terrain_at(x - 1, y).unwrap_or_default(),
                self.get_terrain_at(x, y).unwrap_or_default(),
                self.get_terrain_at(x + 1, y).unwrap_or_default(),
            ],
            [
                self.get_terrain_at(x - 1, y + 1).unwrap_or_default(),
                self.get_terrain_at(x, y + 1).unwrap_or_default(),
                self.get_terrain_at(x + 1, y + 1).unwrap_or_default(),
            ],
        ]
    }

    /// Get the 9 terrain tiles around the given [RoomPos] in the room.
    /// Note the result will be inaccurate if the given RoomPos is not in this room.
    pub fn get_terrain_area_at_rp(&self, pos: RoomPos) -> [[TerrainTile; 3]; 3] {
        self.get_look_terrain_area_at(pos.pos_x, pos.pos_y)
    }

    /// Get the terrain tile at the given coordinates in the room.
    ///
    /// ## Errors
    ///
    /// Returns error if the provided coordinate is out of range (if either coordinate is >= 50).
    pub fn get_terrain_at(&self, x: u8, y: u8) -> Result<TerrainTile, Error> {
        ensure!(x < 50, "X coordinate of terrain tile out of range: {}", x);
        ensure!(y < 50, "Y coordinate of terrain tile out of range: {}", y);
        let byte_offset = (x as usize * 2 + y as usize * 100) / 8;
        let bit_offset = (x as usize * 2 + y as usize * 100) % 8;
        let data = (self.terrain[byte_offset] << bit_offset & 0b11000000) >> 6;
        Ok(TerrainTile::from(data))
    }

    /// Get the terrain tile at the given [RoomPos] in this room.
    /// Note the result will be inaccurate if the given RoomPos is not in this room.
    ///
    /// ## Errors
    ///
    /// See [Self::get_terrain_at].
    pub fn get_terrain_at_rp(&self, pos: RoomPos) -> Result<TerrainTile, Error> {
        self.get_terrain_at(pos.pos_x, pos.pos_y)
    }

    /// Get the coordinates of the room with respect to the top-left corner,
    /// in (x, y) format where (0,0) is the top-leftmost room, from a room name.
    ///
    /// ## Arguments
    ///
    /// - `name`: The room name, formatted as `<E/W><X><N/S><Y>`. For example, "E34N12", or "W4S0".
    ///   Note that E0S0, E0N0, W0S0, and W0N0 are not the same room, they are adjascent rooms in the center of the map.
    pub fn pos_from_name(name: &str) -> Result<(u8, u8), Error> {
        let mut chars = name.chars();
        let ret0: u8;
        let ns;

        // -------------------------------- East / West
        let ew = chars.next();
        ensure!(
            ew.is_some(),
            "Failed to parse room name {}: expected E or W in position 0, got EOF",
            name
        );
        let ew = ew.unwrap();
        ensure!(
            ew == 'E' || ew == 'W',
            "Failed to parse room name {}: expected E or W in position 0, got {}",
            name,
            ew
        );

        // -------------------------------- First digit
        let n1 = chars.next();
        ensure!(
            n1.is_some(),
            "Failed to parse room name {}: expected digit in position 1, got EOF",
            name
        );
        let n1 = n1.unwrap();
        ensure!(
            n1.is_numeric(),
            "Failed to parse room name {}: expected digit in position 1, got {}",
            name,
            n1
        );

        // -------------------------------- Second digit -OR- North / South
        let n2 = chars.next();
        ensure!(
            n2.is_some(),
            "Failed to parse room name {}: expected digit or N or S in position 2, got EOF",
            name
        );

        let n2 = n2.unwrap();

        if n2.is_numeric() {
            let dig1 = n1.to_digit(10).unwrap(); // safe unwrap
            let dig2 = n2.to_digit(10).unwrap(); // safe unwrap

            ret0 = (dig1 * 10 + dig2) as u8;

            if let Some(n3) = chars.next() {
                ns = n3;
            } else {
                bail!(
                    "Failed to parse room name {}: expected N or S in position 3, got EOF",
                    name
                );
            }
            ensure!(
                ns == 'N' || ns == 'S',
                "Failed to parse room name {}: expected N or S in position 3, got {}",
                name,
                ns
            );
        } else {
            ret0 = n1.to_digit(10).unwrap() as u8; // safe unwrap

            ns = n2;
            ensure!(
                ns == 'N' || ns == 'S',
                "Failed to parse room name {}: expected N or S in position 2, got {}",
                name,
                ns
            );
        }

        // -------------------------------- Final digits

        let remainder_str: String = chars.collect();

        let remainder = str::parse::<u8>(&remainder_str);
        ensure!(
            remainder.is_ok(),
            "Failed to parse room name {}: remainder ({}) is not number: {}",
            name,
            remainder_str,
            remainder.err().unwrap()
        );
        let remainder = remainder.unwrap();

        let ret0 = match ew {
            'E' => ret0 + 61,
            'W' => 60 - ret0,
            _ => 0,
        };

        let ret1 = match ns {
            'S' => remainder + 61,
            'N' => 60 - remainder,
            _ => 0,
        };

        Ok((ret0, ret1))
    }

    /// Fill in the terrain data for this room based on a terrain string, imported from a JSON file.
    ///
    /// See [crate::GameState::load_rooms_from_json] for more information on the format of the string.
    pub fn terrain_from_string(&mut self, terrain: &str) -> Result<(), Error> {
        let mut chars = terrain.chars();
        let mut dat = 0u8;
        let mut idx = 0;
        let mut shifts = 0;
        for _y in 0..50 {
            for _x in 0..50 {
                dat <<= 2;
                shifts += 1;
                let t = chars.next();
                ensure!(t.is_some(), "Expected more terrain data, got EOF");
                let t = t.unwrap();
                ensure!(
                    t == '0' || t == '1' || t == '2',
                    "Expected 0, 1, or 2, got {}.",
                    t
                );
                dat |= t.to_digit(10).unwrap() as u8; // Safe unwrap
                if shifts >= 4 {
                    self.terrain[idx] = dat;
                    shifts = 0;
                    idx += 1;
                    dat = 0;
                }
            }
        }
        Ok(())
    }
}
