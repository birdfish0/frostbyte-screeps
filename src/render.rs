//! Provides the [RenderState] struct and functions and enums related to the render thread.

use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};
use log::info;
use macroquad::prelude::*;
use macroquad::text::{Font, load_ttf_font};

use crate::consts::*;
use crate::utils::mouse::mouse_within;
use crate::{game::game_state::GameState, utils::recursive_avg::RecursiveAvg};

#[derive(PartialEq, Eq, Clone, Debug)]
/// A small message sent from the render thread to the server thread.
/// Used to update properties pertaining to the process that calls [GameState::tick] or request a full game state update.
///
/// Note that full game state overrides are sent through their own channel, to keep these messages small in size.
pub enum RenderToServerThreadMsg {
    /// Stop the server loop at the end of the next tick.
    Quit,
    /// Change [GameState::paused] of the server side game state object.
    Pause(bool),
    /// Change [GameState::force_tick_delay] of the server side game state object.
    ForceTickDelay(u32),
    /// Call [GameState::load_rooms_from_json] on the server side game state object with the provided string as argument.
    LoadMapFile(String),
    /// Request that the server send its current [GameState] down the `tx_state` channel passed into [`server::run_loop`][crate::server::run_loop].
    RequestFullState,
}

/// Object containing the current state of the UI and the rendering thread.
///
/// Contains the render-thread version of the [GameState] object, which will not be ticked.
#[derive(Clone, Debug)]
pub struct RenderState {
    /// Render-thread version of the [GameState] object, for drawing reference only. The [`tick`][GameState::tick] function will not be called on it.
    pub game_state: GameState,
    /// The amount of time, in seconds, to wait in between each request of a game state update from the server thread (using [RenderToServerThreadMsg::RequestFullState]).
    pub game_state_update_interval: f32,
    /// Time the last game state update was requested.
    pub last_game_state_update: Instant,
    /// Time the last frame ended.
    pub last_frame_end: Instant,
    /// A [RecursiveAvg] struct that stores the frame times, used in [Self::draw_ui] to get the average fps.
    pub frame_durations: RecursiveAvg,
    /// The font used to draw the UI and other text. Defaults to `assets/CutiveMono-Regular.ttf`.
    pub font: Font,
    /// Horizontal offset of the world map, before scaling, from the position where its top-left corner is positioned at the top-left of the window.
    pub pan_offset_x: f32,
    /// Horizontal offset of the world map, before scaling, from the position where its top-left corner is positioned at the top-left of the window.
    pub pan_offset_y: f32,
    /// When panning the world map, stores the mouse position when the dragging action began. Is [None] when not panning world map.
    pub mouse_drag_begin: Option<Vec2>,
    /// Set to `true` for the frame when a new game state update is received from the server thread. Used in [Self::draw_ui] to draw the update indicator.
    pub got_game_state: bool,
    /// Amount of time it took to draw the last frame, used in [Self::draw_ui] to calculate fps.
    pub latest_frame_duration: Duration,
}

impl RenderState {
    /// Construct a new [RenderState]. Uses the `assets/CutiveMono-Regular.ttf` font by default.
    pub async fn new() -> Self {
        Self {
            game_state: GameState::new(),
            game_state_update_interval: 0.1,
            last_game_state_update: Instant::now(),
            last_frame_end: Instant::now(),
            frame_durations: RecursiveAvg::new(vec![50, 20, 10], false).unwrap_or_else(|err| {
                error!("Failed to create new RecursiveAvg: {}", err);
                Default::default()
            }),
            font: load_ttf_font("assets/CutiveMono-Regular.ttf")
                .await
                .unwrap(),
            pan_offset_x: 0.,
            pan_offset_y: 0.,
            mouse_drag_begin: None,
            latest_frame_duration: Duration::from_nanos(0),
            got_game_state: false,
        }
    }

    /// Draw the terrain of the room located at `(posx, posy)`, shifted by [`Self::pan_offset_x`] and [`Self::pan_offset_y`].
    ///
    /// This should be called before drawing the contents of the room.
    pub fn draw_room_terrain(&self, posx: u8, posy: u8) {
        todo!("{} {}", posx, posy);
    }

    /// Draw the UI onto the screen.
    ///
    /// This should be called after drawing all world objects.
    ///
    /// ## Arguments
    ///
    /// - `mouse_eaten`: Mutable bool reference, will be set to true if
    ///   the mouse is hovering over a UI element and therefore should not be used to zoom/pan the world.
    pub fn draw_ui(&self, mouse_eaten: &mut bool) {
        let top_bar_height = 30.0;
        draw_rectangle(0.0, 0.0, screen_width(), top_bar_height, UI_HIGH_BG_COLOR);
        draw_text_ex(
            format!(
                "tick {:0>12} | {:0>12}ns/t ({:0>8}tps) | {:0>3}fps (avg. {:0>3})",
                self.game_state.tick,
                self.game_state.tick_durations.get(),
                1000000000 / self.game_state.tick_durations.get().max(1),
                1000000000 / self.latest_frame_duration.as_nanos().max(1),
                1000000000 / self.frame_durations.get().max(1)
            ),
            20.0,
            20.0,
            TextParams {
                font_size: 40,
                color: WHITE,
                font_scale: 0.5,
                font: Some(&self.font),
                ..Default::default()
            },
        );
        if self.got_game_state {
            draw_rectangle(0.0, 0.0, 5.0, 5.0, GREEN);
        }

        if mouse_within(0.0, 0.0, screen_width(), top_bar_height) {
            *mouse_eaten = true;
        }
    }

    /// Run the main rendering loop. This will call the relevant draw functions to render the game.
    pub async fn render_loop(
        &mut self,
        tx: Sender<RenderToServerThreadMsg>,
        rx_state: Receiver<GameState>,
        #[allow(unused, reason = "will use later")] tx_state_override: Sender<GameState>,
    ) {
        loop {
            self.latest_frame_duration = self.last_frame_end.elapsed();
            self.frame_durations
                .add(self.latest_frame_duration.as_nanos());

            if self.last_game_state_update.elapsed().as_secs_f32() > self.game_state_update_interval
            {
                self.last_game_state_update = Instant::now();
                tx.send(RenderToServerThreadMsg::RequestFullState)
                    .unwrap_or_else(|err| {
                        error!("Failed to request server game state: {}", err);
                    });
            }

            if !rx_state.is_empty() {
                match rx_state.recv() {
                    Ok(state) => {
                        self.game_state = state;
                        self.game_state.is_server = false;
                        self.got_game_state = true;
                    }
                    Err(err) => {
                        self.got_game_state = false;
                        error!("Error receiving game state from server thread: {}", err);
                    }
                }
            } else {
                self.got_game_state = false;
            }

            let mut mouse_eaten = false;

            clear_background(WORLD_BG_COLOR);

            draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
            draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, BLACK);
            draw_arc(
                screen_width() - 30.0,
                screen_height() - 30.0,
                64,
                14.0,
                0.0,
                5.0,
                90.0,
                YELLOW,
            );

            self.draw_ui(&mut mouse_eaten);

            self.last_frame_end = Instant::now();

            if macroquad::input::is_quit_requested() {
                break;
            }

            next_frame().await
        }

        info!("Render loop stopped");

        tx.send(RenderToServerThreadMsg::Quit).unwrap_or_else(|err| {
            error!("Failed to send stop command to server thread: {}. You may have to force-close the program.", err);
        });
    }
}
