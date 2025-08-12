use std::collections::HashMap;

use winit::{keyboard::KeyCode};

pub struct Input {
    pub keyboard_input: KeyboardInput
}

impl Input {
    pub(crate) fn new() -> Self {
        Self {
            keyboard_input: KeyboardInput::new()
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum KeyboardKeyState {
    Pressed,
    Released,
    Idle,
}


#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum KeyboardKey {
    Code(KeyCode),
    Symbol(char)
}

pub struct KeyboardInput {
    keys_state: HashMap<KeyCode, KeyboardKeyState>,
    symbols_to_codes: HashMap<char, KeyCode>
}


impl KeyboardInput {
    fn new() -> Self {
        Self {
            keys_state: HashMap::new(),
            symbols_to_codes: HashMap::new()
        }
    }

    pub(crate) fn set_key_state(&mut self, key_code: KeyCode, key_symbol: Option<char>, state: KeyboardKeyState) {
        self.keys_state.insert(key_code, state);
        if let Some(sym) = key_symbol {
            self.symbols_to_codes.insert(sym, key_code);
        }
    }

    pub(crate) fn set_released_keys_to_idle(&mut self) {
        self.keys_state
            .values_mut()
            .filter(|state| **state == KeyboardKeyState::Released)
            .for_each(|state| *state = KeyboardKeyState::Idle);
    }

    pub fn is_key_pressed(&self, key: KeyboardKey) -> bool {
        self.check_key_state(key, KeyboardKeyState::Pressed)
    }

    pub fn is_key_released(&self, key: KeyboardKey) -> bool {
        self.check_key_state(key, KeyboardKeyState::Released)
    }


    fn check_key_state(&self, key: KeyboardKey, key_state: KeyboardKeyState) -> bool {
        let key_code = match key {
            KeyboardKey::Code(key_code) => Some(key_code),
            KeyboardKey::Symbol(symbole) => self.symbols_to_codes.get(&symbole).copied(),
        };


        key_code.and_then(|key_code|
            self.keys_state
                .get(&key_code)
                .map(|state| *state == key_state)
            )
            .unwrap_or(false)
    }
}