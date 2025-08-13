
use navagfx_engine::application::{Application, ApplicationSettings};

mod game;
mod physics;

fn main() {

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
