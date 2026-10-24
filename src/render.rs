use std::time::Instant;

use crossbeam_channel::{Receiver, Sender};
use macroquad::prelude::*;
use macroquad::text::{Font, load_ttf_font};

use crate::consts::*;
use crate::{game::game_state::GameState, utils::recursive_avg::RecursiveAvg};

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

pub struct RenderState {
    pub game_state: GameState,
    pub game_state_update_interval: f32,
    pub last_game_state_update: Instant,
    pub last_frame_end: Instant,
    pub frame_durations: RecursiveAvg,
    pub font: Font,
}

impl RenderState {
    pub async fn new() -> Self {
        Self {
            game_state: GameState::new(),
            game_state_update_interval: 0.1,
            last_game_state_update: Instant::now(),
            last_frame_end: Instant::now(),
            frame_durations: RecursiveAvg::new(vec![50, 20, 10], false),
            font: load_ttf_font("assets/CutiveMono-Regular.ttf")
                .await
                .unwrap(),
        }
    }

    pub async fn render_loop(
        &mut self,
        tx: Sender<RenderToServerThreadMsg>,
        rx_state: Receiver<GameState>,
        tx_state_override: Sender<GameState>,
    ) {
        loop {
            let frame_duration = self.last_frame_end.elapsed();
            self.frame_durations.add(frame_duration.as_nanos());

            if self.last_game_state_update.elapsed().as_secs_f32() > self.game_state_update_interval
            {
                self.last_game_state_update = Instant::now();
                tx.send(RenderToServerThreadMsg::RequestFullState);
            }

            let got_game_state;
            if !rx_state.is_empty() {
                match rx_state.recv() {
                    Ok(state) => {
                        self.game_state = state;
                        got_game_state = true;
                    }
                    Err(err) => {
                        got_game_state = false;
                        error!("Error receiving game state from server thread: {}", err);
                    }
                }
            } else {
                got_game_state = false;
            }

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
            draw_rectangle(0.0, 0.0, screen_width(), 30.0, UI_HIGH_BG_COLOR);
            draw_text_ex(
                format!(
                    "tick {:0>12} | {:0>12}ns/t ({:0>8}tps) | {:0>3}fps (avg. {:0>3})",
                    self.game_state.tick,
                    self.game_state.tick_durations.get(),
                    1000000000 / self.game_state.tick_durations.get().max(1),
                    1000000000 / frame_duration.as_nanos().max(1),
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
            if got_game_state {
                draw_rectangle(0.0, 0.0, 5.0, 5.0, GREEN);
            }

            self.last_frame_end = Instant::now();

            next_frame().await
        }

        tx.send(RenderToServerThreadMsg::Quit);
    }
}
