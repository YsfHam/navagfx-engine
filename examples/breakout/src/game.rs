use navagfx_engine::{application::input::{Input, KeyboardKey}, export::application_export::KeyCode, graphics::renderer2d::Renderer2D};

use navagfx_engine::{application::{event::{ApplicationEvent, ApplicationSignal}, ApplicationHandler}, assets::AssetsManagerRef, export::graphics_export::SurfaceError, graphics::GraphicsContext};

use crate::game::game_state::{GameState, LevelData};


mod game_state;


pub struct GameApp {
    renderer: Renderer2D,

    game_state: GameState
}

impl GameApp {    
}

impl ApplicationHandler for GameApp {
    fn init(context: &GraphicsContext, assets_manager: AssetsManagerRef) -> Self {
        let renderer = Renderer2D::new(context, assets_manager);
        
        let level_data = LevelData::load_from_file("assets/two.lvl");
        Self {
            renderer,
            game_state: GameState::new(
                context.config.width as f32,
                context.config.height as f32,
                level_data
            )
        }
    }

    fn update(&mut self, dt: f32) -> ApplicationSignal {
        self.game_state.update(dt)
    }

    fn draw(&mut self, context: &GraphicsContext) -> Result<(), SurfaceError> {
        self.game_state.draw(&mut self.renderer);
        self.renderer.submit(context)
    }

    fn handle_event(&mut self, event: ApplicationEvent) -> ApplicationSignal {
        self.game_state.handle_event(event)
    }
    
    fn handle_input(&mut self, input: &Input) -> ApplicationSignal {

        if input.keyboard_input.is_key_released(KeyboardKey::Code(KeyCode::Escape)) {
            return ApplicationSignal::Exit;
        }

        self.game_state.handle_input(input)
    }
}
