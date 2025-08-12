
use navagfx_engine::application::Application;

mod game;

fn main() {

    Application
        ::<game::GameApp>
        ::new()
        .run()
}
