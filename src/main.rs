use macroquad::miniquad::conf::Icon;
use macroquad::prelude::*;
use std::time::{Instant};
use std::thread;

use crate::util::RecursiveAvg;

mod util;
mod game;

fn window_conf() -> Conf {
    Conf {
         window_title: "Frostbyte Screeps Client".to_owned(),
         icon: Some(icon_from!("../assets/icon_big.bmp", "../assets/icon_med.bmp", "../assets/icon_small.bmp")),
         ..Default::default()
     }
 }

#[macroquad::main(window_conf)]
async fn main() {
    // set_default_filter_mode(FilterMode::Nearest);
    let font = load_ttf_font("assets/CutiveMono-Regular.ttf").await.unwrap();
    
    let mut start_time = Instant::now();

    loop {
        let end_time = Instant::now();
        let frame_duration = end_time.duration_since(start_time);

        clear_background(BLACK);

        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);
        draw_rectangle(0.0, 0.0, screen_width(), 30.0, DARKGRAY);
        draw_text_ex(
            &format!("tick XXXXXXXXXXXX | {}ns/t ({}tps) | {}fps", frame_duration.as_nanos(), 
                                                    1000000000 / frame_duration.as_nanos().max(1),
                                                    1000000000 / frame_duration.as_nanos().max(1)),
            20.0, 20.0, TextParams {
            font_size: 40,
            color: WHITE,
            font_scale: 0.5,
            font: Some(&font),
            ..Default::default()
        });

        start_time = Instant::now();
        
        next_frame().await
    }
}
