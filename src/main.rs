#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
#![allow(unused)]

use crossbeam_channel::{Receiver, Sender, bounded, unbounded};
use log::{error, info};
use log4rs::config::Deserializers;
use macroquad::prelude::*;
use std::fs;
use std::io::Read;
use std::thread::{self, panicking};
use std::time::{Duration, Instant};

use crate::consts::*;
use crate::game::Creep;
use crate::game::game_state::RenderToServerThreadMsg::LoadMapFile;
use crate::game::game_state::{GameState, RenderToServerThreadMsg};
use crate::utils::event_queue::{self, EventQueue};
use crate::utils::recursive_avg::RecursiveAvg;

mod consts;
mod game;
mod server;
mod utils;

fn window_conf() -> Conf {
    Conf {
        window_title: "Frostbyte Screeps Client".to_owned(),
        icon: Some(icon_from!(
            "../assets/icon_big.bmp",
            "../assets/icon_med.bmp",
            "../assets/icon_small.bmp"
        )),
        high_dpi: true,
        window_width: 1300,
        window_height: 800,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let config = log4rs::config::load_config_file("logging.yaml", Deserializers::new())
        .expect("Invalid logger configuration");
    log4rs::init_config(config);

    info!("Logger initialized");

    let font = load_ttf_font("assets/CutiveMono-Regular.ttf")
        .await
        .unwrap();

    let mut start_time = Instant::now();
    let mut durations = RecursiveAvg::new(vec![50, 20, 10], false);

    info!("Setup channels");

    let (tx_state, rx_state) = bounded::<GameState>(1);
    let (tx, rx_srv) = unbounded::<RenderToServerThreadMsg>();
    let (tx_main_state, rx_srv_state) = bounded::<GameState>(1);

    info!("Setup server thread");

    let server_thread = thread::spawn(move || server::run_loop(rx_srv, tx_state, rx_srv_state));

    info!("Loading map data");

    tx.send(LoadMapFile("assets/mmo-shard3.json".to_string()));

    info!("Setup render-thread gamestate");

    let mut render_gamestate = GameState::new();

    let mut game_state_update_interval: f32 = 0.1;
    let mut last_game_state_update: Instant = Instant::now();

    info!("Init done");

    loop {
        let end_time = Instant::now();
        let frame_duration = end_time.duration_since(start_time);
        durations.add(frame_duration.as_nanos());

        if last_game_state_update.elapsed().as_secs_f32() > game_state_update_interval {
            last_game_state_update = Instant::now();
            tx.send(RenderToServerThreadMsg::RequestFullState);
        }

        let got_game_state;
        if !rx_state.is_empty() {
            match rx_state.recv() {
                Ok(state) => {
                    render_gamestate = state;
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
                render_gamestate.tick,
                render_gamestate.tick_durations.get(),
                1000000000 / render_gamestate.tick_durations.get().max(1),
                1000000000 / frame_duration.as_nanos().max(1),
                1000000000 / durations.get().max(1)
            ),
            20.0,
            20.0,
            TextParams {
                font_size: 40,
                color: WHITE,
                font_scale: 0.5,
                font: Some(&font),
                ..Default::default()
            },
        );
        if got_game_state {
            draw_rectangle(0.0, 0.0, 5.0, 5.0, GREEN);
        }

        start_time = Instant::now();

        next_frame().await
    }

    tx.send(RenderToServerThreadMsg::Quit);
    server_thread.join();
}
