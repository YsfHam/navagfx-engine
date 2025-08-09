use winit::{event::{ElementState, KeyEvent, WindowEvent}, keyboard::{Key, PhysicalKey}};

pub use winit::keyboard::KeyCode;


#[derive(Debug)]
pub enum ApplicationEvent {
    Resized {width: u32, height: u32},

    KeyPressed {key_info: KeyInfo, repeat: bool},
    KeyReleased(KeyInfo),
}


#[derive(Debug)]
pub struct KeyInfo {
    pub physical_key_code: KeyCode,
    pub symbol: Option<char>,
}

impl KeyInfo {
    fn new(physical_key_code: KeyCode, symbol: Option<char>) -> Self {
        Self {
            physical_key_code,
            symbol
        }
    }

    pub fn is_key_code(&self, key_code: KeyCode) -> bool {
        self.physical_key_code == key_code
    }

    pub fn is_char(&self, sym: char) -> bool {
        self.symbol.is_some_and(|s| s == sym)
    }
}


impl ApplicationEvent {
    pub fn from_window_event(event: WindowEvent) -> Option<Self> {
        match event {
            WindowEvent::Resized(size) => {
                Some(Self::Resized { width: size.width, height: size.height })
            }

            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(key),
                    logical_key,
                    state,
                    repeat,
                    ..
                },
                ..
            } => {

                let key_info = if let Key::Character(sym_str) = logical_key {
                    KeyInfo::new(key, Some(sym_str.chars().next().unwrap()))
                }
                else {
                    KeyInfo::new(key, None)
                };

                let ev = match state {
                    ElementState::Pressed => Self::KeyPressed { key_info, repeat},
                    ElementState::Released => Self::KeyReleased(key_info),
                };

                Some(ev)
            }

            _ => None
        }
    }
}


pub enum ApplicationSignal {
    Exit,
    Continue,
}