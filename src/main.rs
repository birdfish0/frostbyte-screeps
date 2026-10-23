#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
#![allow(unused)]

use log::{error, info};
use log4rs::config::Deserializers;
use crossbeam_channel::{Receiver, Sender, bounded, unbounded};
use macroquad::prelude::*;
use std::thread::{self, panicking};
use std::time::{Duration, Instant};

use crate::consts::*;
use crate::game::game_state::{GameState, RenderToServerThreadMsg};
use crate::utils::event_queue::{self, EventQueue};
use crate::utils::recursive_avg::RecursiveAvg;

mod consts;
mod game;
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
    let config = log4rs::config::load_config_file("logging.yaml", Deserializers::new()).expect("Invalid logger configuration");
    log4rs::init_config(config);

    debug!("Logger initialized");

    let font = load_ttf_font("assets/CutiveMono-Regular.ttf")
        .await
        .unwrap();

    let mut start_time = Instant::now();
    let mut durations = RecursiveAvg::new(vec![20, 5], false);

    debug!("Setup channels");
    
    let (tx_state, rx_state) = bounded::<GameState>(1);
    let (tx, rx_srv) = unbounded::<RenderToServerThreadMsg>();
    let (tx_main_state, rx_srv_state) = bounded::<GameState>(1);

    debug!("Setup server thread");
    thread::spawn(move || {
        let mut game_state = GameState::new();
        let mut event_queue = EventQueue::new();
        let mut stop = false;
        while !stop {
            while !rx_srv.is_empty() {
                match rx_srv.recv() {
                    Ok(msg) => {
                        match msg {
                            RenderToServerThreadMsg::Quit => {
                                stop = true;
                            },
                            RenderToServerThreadMsg::Pause(paused) => {
                                game_state.paused = paused;
                            },
                            RenderToServerThreadMsg::ForceTickDelay(delay) => {
                                game_state.force_tick_delay = delay;
                            },
                            RenderToServerThreadMsg::RequestFullState => {
                                tx_state.send(game_state.clone());
                            }
                        }
                    }
                    Err(err) => {
                        error!("Failed to read message from render thread: {}", err);
                    }
                }
            }
            game_state.tick(&mut event_queue);
            if game_state.force_tick_delay > 0 {
                thread::sleep(Duration::from_nanos(game_state.force_tick_delay as u64));
            }
        }
    });

    info!("Init done");

    loop {
        let end_time = Instant::now();
        let frame_duration = end_time.duration_since(start_time);
        durations.add(frame_duration.as_nanos());

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
                "tick XXXXXXXXXXXX | {:0>12}ns/t ({:0>4}tps) | {:0>3}fps",
                frame_duration.as_nanos(),
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

        start_time = Instant::now();

        next_frame().await
    }
}
