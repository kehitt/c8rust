// Input
const KEYPAD_SIZE: usize = 16;

pub struct KeypadState {
    pub state: [bool; KEYPAD_SIZE],
}

impl KeypadState {
    pub fn new() -> Self {
        Self {
            state: [false; KEYPAD_SIZE],
        }
    }
}
