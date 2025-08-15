#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")]


use navagfx_engine::application::{Application, ApplicationSettings};

mod game;
mod physics;

fn main() {

    #[cfg(feature = "console")]
    simple_logger::init().unwrap();

    let settings = ApplicationSettings {
        window_title: "Breakout Game",
        window_width: 800,
        window_height: 600,
        window_resizable: false,
    };

    Application
        ::<game::GameApp>
        ::new(settings)
        .run()
}
