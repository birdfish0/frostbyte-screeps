use std::fmt::Display;

use crate::game::room::Room;

#[derive(PartialEq, Eq, Copy, Clone, Debug, Default)]
pub struct RoomPos {
    pub room_x: u8,
    pub room_y: u8,
    pub pos_x: u8,
    pub pos_y: u8,
}

impl RoomPos {
    /// Get the Chebyshev distance from this RoomPos to another point in the same room.
    pub fn get_range_to(&self, x: u8, y: u8) -> u8 {
        self.pos_x.abs_diff(x).max(self.pos_y.abs_diff(y))
    }
}

impl Display for RoomPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RoomPos<{}>({}, {})",
            Room::name_from_pos_checked(self.room_x, self.room_y),
            self.pos_x,
            self.pos_y
        )
    }
}
