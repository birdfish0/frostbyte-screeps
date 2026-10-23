//! Contains the [Room] struct and its implementation, as well as functions related to rooms.

use anyhow::Error;

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
    pub fn pos_from_name(name: &str) -> Result<(u8, u8), Error> {
        let mut chars = name.chars();
        let mut ret0: u8 = 0;
        let mut ns = 'N';
        loop {
            // -------------------------------- East / West
            let ew = chars.next();
            if ew.is_none() {
                break;
            }
            let ew = ew.unwrap();
            if ew != 'E' && ew != 'W' {
                break;
            }

            // -------------------------------- First digit
            let n1 = chars.next();
            if n1.is_none() {
                break;
            }
            let n1 = n1.unwrap();
            if !n1.is_numeric() {
                break;
            }

            // -------------------------------- Second digit -OR- North / South
            let n2 = chars.next();
            if n2.is_none() {
                break;
            }
            let n2 = n2.unwrap();

            if n2.is_numeric() {
                let dig1 = n1.to_digit(10).unwrap(); // safe unwrap
                let dig2 = n2.to_digit(10).unwrap(); // safe unwrap

                ret0 = (dig1 * 10 + dig2) as u8;

                if let Some(n3) = chars.next() {
                    ns = n3;
                }
                else {
                    break;
                }

            } else {
                ret0 = n1.to_digit(10).unwrap() as u8; // safe unwrap

                ns = n2;
            }
            if ns != 'N' && ns != 'S' {
                break;
            }

            // -------------------------------- Final digits

            let mut remainder: String = chars.collect();

            let remainder = u8::from_str_radix(&remainder, 10);
            if remainder.is_err() { break };
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

        Err(Error::msg(format!("Failed to parse room name: {}", name)))
    }
}
