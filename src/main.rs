#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use macroquad::prelude::*;
use std::time::Instant;

use crate::consts::*;
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
    let font = load_ttf_font("assets/CutiveMono-Regular.ttf")
        .await
        .unwrap();

    let mut start_time = Instant::now();
    let mut durations = RecursiveAvg::new(vec![20, 5], false);

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
            &format!(
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
