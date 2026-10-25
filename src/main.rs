#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
#![allow(dead_code, reason = "will use it later")]

use crossbeam_channel::{bounded, unbounded};
use log::{debug, error, info};
use log4rs::config::Deserializers;
use macroquad::prelude::*;
use std::thread;

use crate::game::game_state::GameState;
use crate::render::RenderState;
use crate::render::{RenderToServerThreadMsg::LoadMapFile, *};
use crate::utils::event_queue;

mod consts;
mod game;
mod render;
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
        window_width: 1920,
        window_height: 1080,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let config = log4rs::config::load_config_file("logging.yaml", Deserializers::new())
        .expect("Invalid logger configuration");
    log4rs::init_config(config).expect("Could not initialize logger");

    macroquad::input::prevent_quit();

    debug!("Logger initialized");

    info!("Begin init");

    debug!("Setup channels");

    let (tx_state, rx_state) = bounded::<GameState>(1);
    let (tx, rx_srv) = unbounded::<RenderToServerThreadMsg>();
    let (tx_state_override, rx_state_override) = bounded::<GameState>(1);

    debug!("Setup server thread");

    let server_thread = thread::Builder::new()
        .name("server".to_owned())
        .spawn(move || server::run_loop(rx_srv, tx_state, rx_state_override))
        .expect({
            error!("Failed to spawn server thread. Stopping.");
            "Failed to spawn server thread"
        });

    debug!("Requesting map data load");

    tx.send(LoadMapFile("assets/mmo-shard3.json".to_string()))
        .expect("Failed to send LoadMapFile to server thread");

    debug!("Setup render thread");

    let mut render_state = RenderState::new().await;

    info!("Init done");

    render_state
        .render_loop(tx, rx_state, tx_state_override)
        .await;

    if let Err(err) = server_thread.join() {
        error!("Server thread panicked: {:?}", err);
    }

    info!("Stop complete");
}
