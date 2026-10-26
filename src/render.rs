//! Provides the [RenderState] struct and functions and enums related to the render thread.

use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use anyhow::{Error, bail, ensure};
use crossbeam_channel::{Receiver, Sender};
use log::{debug, error, info};
use macroquad::text::{Font, load_ttf_font};
use macroquad::{input, prelude::*};

use crate::consts::*;
use crate::game::room::{Room, TerrainTile};
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
    /// Offset of the world map, before scaling, from the position where its top-left corner is positioned at the top-left of the window.
    pub pan_offset: Vec2,
    /// Scale of the map, as changed by scroll wheel. At 1.0, each room tile is [TILE_SIZE] pixels big.
    /// Their size is multiplied by `view_scale` when being drawn.
    pub view_scale: f32,
    /// When panning the world map, stores the mouse position when the dragging action began. Is [None] when not panning world map.
    pub mouse_drag_begin: Option<Vec2>,
    /// Set to `true` for the frame when a new game state update is received from the server thread. Used in [Self::draw_ui] to draw the update indicator.
    pub got_game_state: bool,
    /// Amount of time it took to draw the last frame, used in [Self::draw_ui] to calculate fps.
    pub latest_frame_duration: Duration,
    /// `true` if the renderer has already requested the game state over the [Sender] and hasn't received a response yet, `false` otherwise.
    /// Used to prevent the renderer from repeatedly requesting gamestate and filling up the channel.
    pub already_requested_gamestate: bool,
    /// `true` if the renderer has modified its [Self::game_state] independently of the server packets, such as when the user makes a change,
    /// in order to reduce visual latency.
    pub predictive_desync: bool,
    /// An atlas containing images of the terrain of rooms, cached for rendering.
    pub terrain_atlas: Box<Texture2D>,
    /// Whether the `terrain_atlas` has been generated or not.
    ///
    /// This is set to `true` on the frame that the terrain altas generation thread completes so the UI drawer can render the warning text,
    /// and the actual population of the [`Self::terrain_atlas`] happens on the next frame.
    pub terrain_atlas_made: bool,
    /// The momentum of the map movement, applied after the mouse is released.
    pub momentum: Vec2,
}

/// Amount the [momentum][RenderState::momentum] of map movement is decayed by every second. (`momentum -= momentum * MOMENTUM_DECAY * frame_delta`)
/// (multiplied by frame delta)
const MOMENTUM_DECAY: f32 = 4.;

impl RenderState {
    /// Construct a new [RenderState]. Uses the `assets/CutiveMono-Regular.ttf` font by default.
    pub async fn new() -> Self {
        Self {
            game_state: GameState::new(),
            game_state_update_interval: 0.1,
            last_game_state_update: Instant::now(),
            last_frame_end: Instant::now(),
            frame_durations: RecursiveAvg::new(vec![10, 5, 5], false).unwrap_or_else(|err| {
                error!("Failed to create new RecursiveAvg: {}", err);
                Default::default()
            }),
            font: load_ttf_font("assets/CutiveMono-Regular.ttf")
                .await
                .unwrap(),
            pan_offset: Vec2 { x: 0., y: 0. },
            view_scale: 1.,
            mouse_drag_begin: None,
            latest_frame_duration: Duration::from_nanos(0),
            got_game_state: false,
            already_requested_gamestate: false,
            predictive_desync: false,
            terrain_atlas: Box::new(Texture2D::empty()),
            terrain_atlas_made: false,
            momentum: Vec2::ZERO,
        }
    }

    /// Generate the image for the terrain (one pixel per tile) in the room at (posx, posy) on the map, using the rooms parameter passed in.
    pub fn room_terrain_image(
        rooms: &Vec<Vec<Room>>,
        posx: usize,
        posy: usize,
    ) -> Result<[u8; 50 * 50 * 4], Error> {
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

        let mut text = [0u8; 50 * 50 * 4];
        for y in 0..50 {
            for x in 0..50 {
                let idx = (x as usize + y as usize * 50) * 4;
                let room = &rooms[posy][posx];
                let color = match room.get_terrain_at(x, y)? {
                    TerrainTile::Plains => Color::from_rgba(0, 0, 0, 0),
                    TerrainTile::Swamp => TERRAIN_SWAMP_COLOR,
                    TerrainTile::Wall => TERRAIN_WALL_COLOR,
                    TerrainTile::Void => bail!(
                        "Unexpected Void tile in room at ({}, {}) at {}, {}",
                        posx,
                        posy,
                        x,
                        y
                    ),
                };
                text[idx] = (color.r * 255.) as u8;
                text[idx + 1] = (color.g * 255.) as u8;
                text[idx + 2] = (color.b * 255.) as u8;
                text[idx + 3] = (color.a * 255.) as u8;
            }
        }
        Ok(text)
    }

    /// Return a value that can be put into [`Self::terrain_atlas`] with room data from [`Self::game_state::rooms`][GameState::rooms].
    pub fn create_terrain_cache(rooms: Vec<Vec<Room>>) -> Vec<u8> {
        info!("Caching terrain images");
        let mut data = vec![0; 50 * MAP_SIZE_X * 50 * MAP_SIZE_Y * 4];
        for y in 0..MAP_SIZE_Y {
            for x in 0..MAP_SIZE_X {
                match Self::room_terrain_image(&rooms, x, y) {
                    Ok(tex) => {
                        for row in 0..50 {
                            data[((row + 50 * y) * (50 * MAP_SIZE_X) + 50 * x) * 4
                                ..((row + 50 * y) * (50 * MAP_SIZE_X) + 50 * x + 50) * 4]
                                .copy_from_slice(tex.split_at(row * 50 * 4).1.split_at(50 * 4).0);
                        }
                    }
                    Err(err) => error!(
                        "Failed drawing terrain image for room at ({}, {}): {}",
                        x, y, err
                    ),
                }
            }
        }

        data
    }

    /// Draw all the terrain, panned and scaled by [`Self::pan_offset`] and [`Self::view_scale`].
    pub fn draw_terrain(&self) {
        let x = self.pan_offset.x * self.view_scale;
        let y = self.pan_offset.y * self.view_scale;
        let map_size = Vec2 {
            x: 50. * self.view_scale * TILE_SIZE * MAP_SIZE_X as f32,
            y: 50. * self.view_scale * TILE_SIZE * MAP_SIZE_Y as f32,
        };
        draw_rectangle(x, y, map_size.x, map_size.y, TERRAIN_PLAINS_COLOR);
        draw_rectangle(
            x,
            y,
            map_size.x,
            TILE_SIZE * self.view_scale,
            ROOM_EDGE_COLOR,
        );
        draw_rectangle(
            x,
            map_size.y - TILE_SIZE * self.view_scale + self.pan_offset.y * self.view_scale,
            map_size.x,
            TILE_SIZE * self.view_scale,
            ROOM_EDGE_COLOR,
        );
        draw_rectangle(
            x,
            y,
            TILE_SIZE * self.view_scale,
            map_size.y,
            ROOM_EDGE_COLOR,
        );
        draw_rectangle(
            map_size.x - TILE_SIZE * self.view_scale + self.pan_offset.x * self.view_scale,
            y,
            TILE_SIZE * self.view_scale,
            map_size.y,
            ROOM_EDGE_COLOR,
        );
        for x in 1..MAP_SIZE_X - 2 {
            draw_rectangle(
                ((x as f32 * 50. - 1.) * TILE_SIZE + self.pan_offset.x) * self.view_scale,
                y,
                TILE_SIZE * self.view_scale * 2.,
                map_size.y,
                ROOM_EDGE_COLOR,
            );
        }
        for y in 1..MAP_SIZE_X - 2 {
            draw_rectangle(
                x,
                ((y as f32 * 50. - 1.) * TILE_SIZE + self.pan_offset.y) * self.view_scale,
                map_size.x,
                TILE_SIZE * self.view_scale * 2.,
                ROOM_EDGE_COLOR,
            );
        }

        draw_texture_ex(
            &self.terrain_atlas,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(map_size),
                source: None,
                rotation: 0.,
                flip_x: false,
                flip_y: false,
                pivot: None,
            },
        );
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
        if self.predictive_desync {
            draw_rectangle(5.0, 0.0, 5.0, 5.0, RED);
        }
        if self.already_requested_gamestate {
            draw_rectangle(0.0, 0.0, 5.0, 5.0, ORANGE);
        }
        if self.got_game_state {
            draw_rectangle(0.0, 0.0, 5.0, 5.0, GREEN);
        }
        if self.game_state.paused {
            draw_rectangle(0.0, 5.0, 5.0, 5.0, ORANGE);
        }

        if mouse_within(0.0, 0.0, screen_width(), top_bar_height) {
            *mouse_eaten = true;
        }

        if self.terrain_atlas_made && self.terrain_atlas.height() == 1. {
            let center = get_text_center(
                "Creating terrain atlas, this may lag a bit...",
                Some(&self.font),
                60,
                0.5,
                0.,
            );
            draw_text_ex(
                "Creating terrain atlas, this may lag a bit...",
                screen_width() / 2. - center.x,
                screen_height() / 2. - center.y,
                TextParams {
                    font_size: 60,
                    color: ORANGE,
                    font_scale: 0.5,
                    font: Some(&self.font),
                    ..Default::default()
                },
            );
        }
    }

    /// Run the main rendering loop. This will call the relevant draw functions to render the game.
    pub async fn render_loop(
        &mut self,
        tx: Sender<RenderToServerThreadMsg>,
        rx_state: Receiver<GameState>,
        #[allow(unused, reason = "will use later")] tx_state_override: Sender<GameState>,
    ) {
        let mut atlasgen: Option<JoinHandle<Vec<u8>>> = None;
        loop {
            self.latest_frame_duration = self.last_frame_end.elapsed();
            self.frame_durations
                .add(self.latest_frame_duration.as_nanos());

            self.last_frame_end = Instant::now();

            if !rx_state.is_empty() {
                match rx_state.recv() {
                    Ok(state) => {
                        self.game_state = state;
                        self.game_state.is_server = false;
                        self.got_game_state = true;
                        self.already_requested_gamestate = false;
                        self.predictive_desync = false;
                    }
                    Err(err) => {
                        self.got_game_state = false;
                        error!("Error receiving game state from server thread: {}", err);
                    }
                }
            } else {
                self.got_game_state = false;
            }

            if !self.already_requested_gamestate
                && self.last_game_state_update.elapsed().as_secs_f32()
                    > self.game_state_update_interval
            {
                self.last_game_state_update = Instant::now();
                tx.send(RenderToServerThreadMsg::RequestFullState)
                    .unwrap_or_else(|err| {
                        error!("Failed to request server game state: {}", err);
                    });
                self.already_requested_gamestate = true;
            }

            if input::is_key_pressed(KeyCode::P) {
                if let Err(err) = tx.send(RenderToServerThreadMsg::Pause(!self.game_state.paused)) {
                    error!("Failed to send pause request to server: {}", err);
                } else {
                    self.game_state.paused = !self.game_state.paused;
                    self.predictive_desync = true;
                }
            }

            if atlasgen.is_none()
                && !self.terrain_atlas_made
                && self.game_state.rooms[0][0].terrain[0] != 0
            {
                debug!("Start atlasgen thread");
                let rooms = self.game_state.rooms.clone();
                match thread::Builder::new()
                    .name("load_terrain_atlas".to_owned())
                    .spawn(|| Self::create_terrain_cache(rooms))
                {
                    Err(err) => error!("Failed to start atlasgen thread: {}", err),
                    Ok(thrd) => atlasgen = Some(thrd),
                }
            } else if let Some(ref thrd) = atlasgen {
                if thrd.is_finished() {
                    if self.terrain_atlas_made {
                        match atlasgen.unwrap().join() {
                            Err(err) => error!("Atlas generation thread panicked: {:?}", err),
                            Ok(atlas) => {
                                debug!("Joining atlasgen thread");
                                self.terrain_atlas = Box::new(Texture2D::from_rgba8(
                                    50 * MAP_SIZE_X as u16,
                                    50 * MAP_SIZE_Y as u16,
                                    &atlas,
                                ));
                                self.terrain_atlas.set_filter(FilterMode::Nearest);
                            }
                        };
                        atlasgen = None;
                    }
                    self.terrain_atlas_made = true;
                }
            }

            let mut mouse_eaten = false;

            clear_background(WORLD_BG_COLOR);

            self.draw_terrain();

            if !mouse_eaten {
                let highlight = ((((Vec2::from(mouse_position()) / self.view_scale
                    - self.pan_offset)
                    / TILE_SIZE)
                    .floor()
                    * TILE_SIZE)
                    + self.pan_offset)
                    * self.view_scale;
                draw_rectangle(
                    highlight.x,
                    highlight.y,
                    TILE_SIZE * self.view_scale,
                    TILE_SIZE * self.view_scale,
                    CURSOR_HIGHLIGHT_COLOR,
                );
            }

            self.draw_ui(&mut mouse_eaten);

            self.pan_and_zoom(mouse_eaten);

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

    pub fn pan_and_zoom(&mut self, mouse_eaten: bool) {
        let mouse_pos = Vec2::from(mouse_position());
        let any_mouse_btn_down = input::is_mouse_button_down(MouseButton::Left)
            || input::is_mouse_button_down(MouseButton::Middle)
            || input::is_mouse_button_down(MouseButton::Right);

        if let Some(begin_pos) = self.mouse_drag_begin {
            let delta = begin_pos - mouse_pos;
            self.momentum = delta / self.view_scale / self.latest_frame_duration.as_secs_f32();
            self.pan_offset -= delta / self.view_scale;
        }
        if !mouse_eaten && any_mouse_btn_down {
            self.mouse_drag_begin = Some(mouse_pos);
        } else {
            self.mouse_drag_begin = None;
            self.pan_offset -= self.momentum * self.latest_frame_duration.as_secs_f32();
            self.momentum -=
                self.momentum * (MOMENTUM_DECAY * self.latest_frame_duration.as_secs_f32()).min(1.);
        }

        if !mouse_eaten {
            let zdelta = input::mouse_wheel().1 / 240.;
            if zdelta != 0. {
                let new_scale = self.view_scale * 2f32.powf(zdelta);
                // Point of interest = point where the mouse is hovering at the time of zoom

                // Map position of point of interest
                let poi_map_pos = mouse_pos / self.view_scale - self.pan_offset;
                // The position of the point of interest on the screen were the new zoom to be applied
                let new_poi_pos = (poi_map_pos + self.pan_offset) * new_scale;
                // Amount of pixels the new map needs to be shifted by to visually be scaled at the mouse position
                let vshift = mouse_pos - new_poi_pos;

                self.pan_offset += vshift / new_scale;
                self.view_scale = new_scale;
            }
        }
    }
}
