//! Provides functions related to the mouse position and changing the cursor.

use macroquad::input::mouse_position;

/// Returns `true` if the mouse is within the specified rectangle, otherwise returns `false`.
///
/// The rectangle has its top-left corner at [x, y] with size [w, h] (width going to the right, height going down)
pub fn mouse_within(x: f32, y: f32, w: f32, h: f32) -> bool {
    let mp = mouse_position();
    x <= mp.0 && mp.0 <= x + w && y <= mp.1 && mp.1 <= y + h
}
