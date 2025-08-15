use navagfx_engine::{application::{input::{Input, KeyboardKey}, GraphicsContextRef}, export::application_export::KeyCode, graphics::renderer2d::Renderer2D};

use navagfx_engine::{application::{event::{ApplicationEvent, ApplicationSignal}, ApplicationHandler}, assets::AssetsManagerRef, export::graphics_export::SurfaceError};

use crate::game::game_state::GameState;


mod game_state;
mod entities;


pub struct GameApp {
    renderer: Renderer2D,

    game_state: GameState

}

impl GameApp {    
}


impl ApplicationHandler for GameApp {
    fn init(context: GraphicsContextRef<'static>, assets_manager: AssetsManagerRef) -> Self {
        
        let context_lock = context.read().unwrap();
        let width = context_lock.config.width;
        let height = context_lock.config.height;
        drop(context_lock);


        let renderer = Renderer2D::new(context.clone(), assets_manager.clone());

        Self {
            renderer,
            game_state: GameState::new(
                width as f32,
                height as f32,
                assets_manager.clone()
            )
        }
    }

    fn update(&mut self, dt: f32) -> ApplicationSignal {
        self.game_state.update(dt)
    }

    fn draw(&mut self) -> Result<(), SurfaceError> {
        self.game_state.draw(&mut self.renderer);
        self.renderer.submit()
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
