//! Provides the [run_loop] function, in charge of all server thread logic.

use std::{fs, io::Read, thread, time::Duration};

use crossbeam_channel::{Receiver, Sender};
use log::{error, info};

use crate::{game::game_state::*, render::RenderToServerThreadMsg, utils::event_queue::EventQueue};

/// Run the server logic in a loop until [RenderToServerThreadMsg::Quit] is received.
///
/// Reads and processes messages from the render thread before each tick.
/// The game state is overwritten before the tick and before any requests are processed if anything is sent on the `rx_state_override` channel.
pub fn run_loop(
    rx: Receiver<RenderToServerThreadMsg>,
    tx: Sender<GameState>,
    rx_state_override: Receiver<GameState>,
) {
    let mut game_state = GameState::new();
    game_state.is_server = true;
    let mut event_queue = EventQueue::new();
    let mut stop = false;
    while !stop {
        while !rx_state_override.is_empty() {
            match rx_state_override.recv() {
                Ok(msg) => {
                    game_state = msg;
                    game_state.is_server = true;
                }
                Err(err) => {
                    error!("Failed to read game state override: {}", err);
                }
            }
        }
        while !rx.is_empty() {
            match rx.recv() {
                Ok(msg) => match msg {
                    RenderToServerThreadMsg::Quit => {
                        stop = true;
                    }
                    RenderToServerThreadMsg::Pause(paused) => {
                        game_state.paused = paused;
                    }
                    RenderToServerThreadMsg::ForceTickDelay(delay) => {
                        game_state.force_tick_delay = delay;
                    }
                    RenderToServerThreadMsg::LoadMapFile(filepath) => {
                        let file = fs::File::open(&filepath);
                        if let Ok(mut file) = file {
                            let mut data = String::new();
                            match file.read_to_string(&mut data) {
                                Err(err) => {
                                    error!("Failed to read map file {}: {}", filepath, err)
                                }
                                Ok(_) => {
                                    game_state
                                        .load_rooms_from_json(&data, &mut event_queue)
                                        .unwrap_or_else(|err| {
                                            error!("Failed to load game data from JSON: {}", err);
                                        });
                                }
                            }
                        } else {
                            error!(
                                "Failed to read map file {}: {}",
                                filepath,
                                file.err().unwrap()
                            );
                        }
                    }
                    RenderToServerThreadMsg::RequestFullState => {
                        tx.send(game_state.clone()).unwrap_or_else(|err| {
                            error!("Failed to send game state to render thread: {}", err);
                        });
                    }
                },
                Err(err) => {
                    error!("Failed to read message from render thread: {}", err);
                }
            }
        }
        game_state.tick(&mut event_queue);
        if game_state.force_tick_delay > 0 {
            thread::sleep(Duration::from_nanos(game_state.force_tick_delay as u64));
        } else if game_state.paused {
            thread::sleep(Duration::from_secs_f32(0.1));
        }
    }
    info!("Server thread stopping");
}
