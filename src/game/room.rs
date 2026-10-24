//! Contains the [Room] struct and its implementation, as well as functions related to rooms.

use anyhow::{Error, anyhow, bail, ensure};

/// Contains data pertaining to a room on the map.
///
/// Does not contain the actual data of most of the objects within the room, only their ID.
#[derive(PartialEq, Eq, Clone, Debug)]
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

impl Default for Room {
    fn default() -> Self {
        Self {
            posx: 0,
            posy: 0,
            creeps: Vec::new(),
            terrain: [0; 50],
            structures: Vec::new(),
            energy_sources: Vec::new(),
            mineral_source: None,
            commodity_source: None,
            construction_sites: Vec::new(),
        }
    }
}

impl Room {
    /// Get the coordinates of the room with respect to the top-left corner,
    /// in (x, y) format where (0,0) is the top-leftmost room, from a room name.
    ///
    /// ## Arguments
    ///
    /// - `name`: The room name, formatted as "<E/W><X><N/S><Y>". For example, "E34N12", or "W4S0".
    /// Note that E0S0, E0N0, W0S0, and W0N0 are not the same room, they are adjascent rooms in the center of the map.
    pub fn pos_from_name(name: &str) -> Result<(u8, u8), Error> {
        let mut chars = name.chars();
        let mut ret0: u8 = 0;
        let mut ns;

        // -------------------------------- East / West
        let ew = chars.next();
        ensure!(
            ew.is_some(),
            "Failed to parse room name {}: expected E or W in position 0, got EOF",
            name
        );
        let ew = ew.unwrap();
        ensure!(
            ew == 'E' && ew == 'W',
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

        let mut remainder_str: String = chars.collect();

        let remainder = u8::from_str_radix(&remainder_str, 10);
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

        return Ok((ret0, ret1));
    }

    /// Fill in the terrain data for this room based on a terrain string, imported from a JSON file.
    ///
    /// See [crate::GameState::load_rooms_from_json] for more information on the format of the string.
    pub fn terrain_from_string(&mut self, terrain: &str) -> Result<(), Error> {
        let mut chars = terrain.chars();
        for y in 0..50 {
            let mut row = 0u128;
            for x in 0..50 {
                let t = chars.next();
                ensure!(t.is_some(), "Expected more terrain data, got EOF");
                let t = t.unwrap();
                ensure!(
                    t == '0' || t == '1' || t == '2',
                    "Expected 0, 1, or 2, got {}.",
                    t
                );
                row |= t.to_digit(10).unwrap() as u128; // Safe unwrap
                row <<= 2;
            }
            row >>= 2;
            self.terrain[y] = row;
        }
        Ok(())
    }
}
