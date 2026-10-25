//! Contains the [Room] struct and its implementation, as well as functions related to rooms.

use std::collections::HashSet;

use anyhow::{Error, bail, ensure};
use seahash::SeaHasher;

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
    pub creeps: HashSet<u64, SeaHasher>,
    /// Terrain of the room, by rows. Each tile is represented by 2 bits,
    /// where `0b00` is plains, `0b01` is wall, `0b10` is swamp.
    ///
    /// This means that the first 100 bits will represent the first row of the terrain (top row),
    /// and bits 101-200 represent the second row, etc.
    pub terrain: [u8; 625],
    /// IDs of [Structure]s in the room.
    pub structures: HashSet<u64, SeaHasher>,
    /// IDs of [EnergySource]s in the room.
    pub energy_sources: HashSet<u64, SeaHasher>,
    /// ID of the [MineralSource] in the room, if there is one.
    pub mineral_source: Option<u64>,
    /// ID of the [CommoditySource] in the room, if there is one.
    pub commodity_source: Option<u64>,
    /// IDs of [ConstructionSite]s in the room.
    pub construction_sites: HashSet<u64, SeaHasher>,
    /// IDs of [Ruin]s in the room.
    pub ruins: HashSet<u64, SeaHasher>,
    /// IDs of [Tombstone]s in the room.
    pub tombstones: HashSet<u64, SeaHasher>,
    /// IDs of [DroppedResource]s in the room.
    pub dropped_resources: HashSet<u64, SeaHasher>,
}

impl Default for Room {
    fn default() -> Self {
        Self {
            posx: 0,
            posy: 0,
            creeps: HashSet::with_hasher(SeaHasher::new()),
            terrain: [0; 625],
            structures: HashSet::with_hasher(SeaHasher::new()),
            energy_sources: HashSet::with_hasher(SeaHasher::new()),
            mineral_source: None,
            commodity_source: None,
            construction_sites: HashSet::with_hasher(SeaHasher::new()),
            ruins: HashSet::with_hasher(SeaHasher::new()),
            tombstones: HashSet::with_hasher(SeaHasher::new()),
            dropped_resources: HashSet::with_hasher(SeaHasher::new()),
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

impl Room {
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
        let byte_offset = (x + y * 50 / 4) as usize;
        let bit_offset = x + y * 50 % 4;
        let data = self.terrain[byte_offset] << bit_offset & 0b11000000;
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
            'N' => remainder + 61,
            'S' => 60 - remainder,
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
