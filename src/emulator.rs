use winit::{
    event::{DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
    window::Window,
};

use crate::{beeper::Beeper, keypad::KeypadState, renderer::Renderer, timing::Timing, vm::VM};

const TICK_RATE_MIN: u64 = 100;
const TICK_RATE_NORMAL: u64 = 250;
const TICK_RATE_FAST: u64 = 500;
const TICK_RATE_MAX: u64 = 1000;

// Instructions per second
const DEFAULT_TICK_RATE: u64 = TICK_RATE_NORMAL;
// Frames per second
const DEFAULT_FRAME_RATE: u64 = 60;

pub struct Emulator {
    renderer: Renderer,
    beeper: Beeper,
    vm: VM,
    keypad: KeypadState,
    timing: Timing,
}

impl Emulator {
    pub fn new(window: &Window) -> Self {
        let renderer = pollster::block_on(Renderer::new(window));
        let mut beeper = Beeper::new();
        beeper.start_stream();
        let vm = VM::new(&[]);
        let keypad = KeypadState::new();
        let timing = Timing::new(DEFAULT_TICK_RATE, DEFAULT_FRAME_RATE);

        Self {
            renderer,
            beeper,
            vm,
            keypad,
            timing,
        }
    }

    pub fn handle_window_event(&mut self, event: WindowEvent) -> Option<ControlFlow> {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                return Some(ControlFlow::Exit);
            }
            WindowEvent::DroppedFile(path_buf) => {
                let rom = std::fs::read(path_buf.into_os_string().to_str().unwrap()).unwrap();
                self.vm = VM::new(&rom);
                self.keypad = KeypadState::new()
            }
            WindowEvent::Resized(physical_size) => self.renderer.on_resize(physical_size),
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                self.renderer.on_resize(*new_inner_size)
            }
            _ => (),
        };

        None
    }

    pub fn handle_device_event(&mut self, event: DeviceEvent) -> Option<ControlFlow> {
        if let DeviceEvent::Key(KeyboardInput {
            state: element_state,
            virtual_keycode: Some(keycode),
            ..
        }) = event
        {
            match element_state {
                ElementState::Pressed => self.on_key_pressed(keycode),
                ElementState::Released => self.on_key_released(keycode),
            }
        }

        None
    }

    pub fn handle_update(&mut self, window: &Window) -> Option<ControlFlow> {
        if self.timing.should_tick() {
            self.vm.tick(&self.keypad);
            self.beeper.set_beeper_active(self.vm.is_beeper_active());
            self.timing.mark_tick()
        }

        if self.timing.should_draw() {
            window.request_redraw();
            self.timing.mark_draw()
        }

        self.timing.try_sleep();

        None
    }

    pub fn handle_redraw(&mut self) -> Option<ControlFlow> {
        if let Some(modification_data) = self.vm.pop_display_modifications() {
            self.renderer.write_display_modifications(modification_data);
        }
        self.renderer.on_redraw();

        None
    }

    fn on_key_pressed(&mut self, keycode: VirtualKeyCode) {
        if let Some(key_idx) = map_key(keycode) {
            self.keypad.state[key_idx as usize] = true;
        } else {
            self.adjust_tickrate(keycode);
        }
    }

    fn on_key_released(&mut self, keycode: VirtualKeyCode) {
        if let Some(key_idx) = map_key(keycode) {
            self.keypad.state[key_idx as usize] = false;
        }
    }

    fn adjust_tickrate(&mut self, keycode: VirtualKeyCode) {
        match keycode {
            VirtualKeyCode::F1 => self.timing.tickrate = TICK_RATE_MIN,
            VirtualKeyCode::F2 => self.timing.tickrate = TICK_RATE_NORMAL,
            VirtualKeyCode::F3 => self.timing.tickrate = TICK_RATE_FAST,
            VirtualKeyCode::F4 => self.timing.tickrate = TICK_RATE_MAX,
            _ => (),
        }
    }
}

fn map_key(scancode: VirtualKeyCode) -> Option<u8> {
    match scancode {
        VirtualKeyCode::Key1 => Some(1),
        VirtualKeyCode::Key2 => Some(2),
        VirtualKeyCode::Key3 => Some(3),
        VirtualKeyCode::Key4 => Some(0xC),

        VirtualKeyCode::Q => Some(4),
        VirtualKeyCode::W => Some(5),
        VirtualKeyCode::E => Some(6),
        VirtualKeyCode::R => Some(0xD),

        VirtualKeyCode::A => Some(7),
        VirtualKeyCode::S => Some(8),
        VirtualKeyCode::D => Some(9),
        VirtualKeyCode::F => Some(0xE),

        VirtualKeyCode::Z => Some(0xA),
        VirtualKeyCode::X => Some(0),
        VirtualKeyCode::C => Some(0xB),
        VirtualKeyCode::V => Some(0xF),

        _ => None,
    }
}
