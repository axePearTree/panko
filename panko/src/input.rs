use crate::Key;

#[derive(Clone, Debug, Default)]
pub struct InputState {
    pub keyboard: KeyboardState,
}

#[derive(Clone, Debug, Default)]
pub struct KeyboardState {
    keys: [KeyState; Key::Count as usize],
}

#[derive(Copy, Clone, Debug, Default)]
pub struct KeyState {
    down: bool,
    just_down: bool,
    just_up: bool,
}

impl KeyboardState {
    pub fn is_key_down(&self, key: Key) -> bool {
        self.keys[key as usize].down
    }

    pub fn is_key_up(&self, key: Key) -> bool {
        !self.keys[key as usize].down
    }

    pub fn is_key_just_down(&self, key: Key) -> bool {
        !self.keys[key as usize].just_down
    }

    pub fn is_key_just_up(&self, key: Key) -> bool {
        !self.keys[key as usize].just_up
    }

    pub(crate) fn clear_memory(&mut self) {
        for key in self.keys.iter_mut() {
            key.just_down = false;
            key.just_up = false;
        }
    }

    pub(crate) fn on_key_down(&mut self, key: Key) {
        self.keys[key as usize].down = true;
        self.keys[key as usize].just_down = true;
        self.keys[key as usize].just_up = false;
    }

    pub(crate) fn on_key_up(&mut self, key: Key) {
        self.keys[key as usize].down = false;
        self.keys[key as usize].just_down = false;
        self.keys[key as usize].just_up = false;
    }
}

