// Memory region sizes
const MEM_SIZE: usize = 4096;
#[allow(dead_code)] // I'll leave those for now
const MEM_SIZE_INT: usize = 0x1FF;
const MEM_SIZE_FONT: usize = 0x50;
#[allow(dead_code)]
const MEM_SIZE_RAM: usize = 0xDFF;

// Memory region starting addresses
#[allow(dead_code)]
const MEM_REGION_INT: u16 = 0x000;
const MEM_REGION_FONT: u16 = 0x050;
const MEM_REGION_RAM: u16 = 0x200;

// Stack
pub const STACK_SIZE: usize = 16;

pub struct Memory {
    memory: [u8; MEM_SIZE],
}

pub struct Stack {
    stack: [u16; STACK_SIZE],
    stack_pointer: usize,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            memory: [0; MEM_SIZE],
        }
    }

    pub fn load_font(&mut self, fontset: &[u8]) -> u16 {
        self.write_region(
            MEM_REGION_FONT as usize,
            MEM_REGION_FONT as usize + MEM_SIZE_FONT,
            fontset,
        );
        MEM_REGION_FONT
    }

    pub fn load_rom(&mut self, rom_data: &[u8]) -> u16 {
        // Roms are stored BE
        for (i, data) in rom_data.iter().enumerate() {
            self.memory[MEM_REGION_RAM as usize + i] = u8::from_be(*data);
        }

        MEM_REGION_RAM
    }

    pub fn get_font_sprite_location(&self, sprite_id: usize) -> u16 {
        MEM_REGION_FONT + (5 * sprite_id) as u16
    }

    pub fn set8(&mut self, address: usize, value: u8) {
        self.memory[address] = value;
    }

    pub fn get8(&self, address: usize) -> u8 {
        self.memory[address]
    }

    pub fn get16(&self, address: usize) -> u16 {
        (self.memory[address] as u16) << 8 | self.memory[address + 1] as u16
    }

    fn write_region(&mut self, start: usize, end: usize, data: &[u8]) {
        self.memory[start..end].copy_from_slice(data);
    }
}

impl Stack {
    pub fn new() -> Self {
        Self {
            stack: [0; STACK_SIZE],
            stack_pointer: 0,
        }
    }

    pub fn push(&mut self, value: u16) {
        self.stack[self.stack_pointer] = value;
        self.stack_pointer += 1
    }

    pub fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer]
    }
}
