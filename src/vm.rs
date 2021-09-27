use rand::Rng;

use crate::display::{DisplayState, ModificationData};
use crate::keypad::KeypadState;
use crate::memory::{Memory, Stack};
use crate::opcode::OpCode;

// Registers
pub const REGISTER_NUM: usize = 16;

const INSTRUCTION_SIZE: u16 = 2;

// Fontset
const FONTSET: [u8; 5 * 16] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct VM {
    memory: Memory,
    registers: [u8; REGISTER_NUM],
    index_register: u16,
    program_counter: u16,
    delay_timer: u8,
    sound_timer: u8,
    stack: Stack,
    display: DisplayState,
    //
    rng: rand::rngs::ThreadRng,
}

#[derive(PartialEq)]
enum InstructionResult {
    Nop,
    Next,
    Skip,
    Jump(u16),
}

impl VM {
    // Pub

    pub fn new(rom_data: &[u8]) -> Self {
        let mut memory = Memory::new();
        memory.load_font(&FONTSET);
        let program_counter = memory.load_rom(rom_data);

        let mut display_state = DisplayState::new();
        display_state.clear(false); // Fill the first frame

        VM {
            memory,
            registers: [0; REGISTER_NUM],
            index_register: 0,
            program_counter,
            delay_timer: 0,
            sound_timer: 0,
            stack: Stack::new(),
            display: display_state,
            rng: rand::thread_rng(),
        }
    }

    pub fn tick(&mut self, keypad: &KeypadState) {
        let opcode = OpCode::from_bytes(self.memory.get16(self.program_counter.into()));

        self.execute(opcode, keypad);

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn pop_display_modifications(&mut self) -> Option<ModificationData> {
        self.display.pop_modifications()
    }

    pub fn is_beeper_active(&self) -> bool {
        self.sound_timer > 0
    }

    // Priv

    #[inline]
    fn execute(&mut self, opcode: OpCode, keypad: &KeypadState) {
        use OpCode::*;
        let result = match opcode {
            NOP() => self.nop(),
            CLS() => self.cls(),
            RET() => self.ret(),
            JP(addr) => self.jp(addr),
            CALL(addr) => self.call(addr),
            SEVB(x, byte) => self.sevb(x.into(), byte),
            SNEVB(x, byte) => self.snevb(x.into(), byte),
            SEVV(x, y) => self.sevv(x.into(), y.into()),
            LDVB(x, byte) => self.ldvb(x.into(), byte),
            ADDVB(x, byte) => self.addvb(x.into(), byte),
            LDVV(x, y) => self.ldvv(x.into(), y.into()),
            ORVV(x, y) => self.orvv(x.into(), y.into()),
            ANDVV(x, y) => self.andvv(x.into(), y.into()),
            XORVV(x, y) => self.xorvv(x.into(), y.into()),
            ADDVV(x, y) => self.addvv(x.into(), y.into()),
            SUBVV(x, y) => self.subvv(x.into(), y.into()),
            SHRVV(x, y) => self.shrvv(x.into(), y.into()),
            SUBNVV(x, y) => self.subnvv(x.into(), y.into()),
            SHLVV(x, y) => self.shlvv(x.into(), y.into()),
            SNEVV(x, y) => self.snevv(x.into(), y.into()),
            LDIA(addr) => self.ldia(addr),
            JPVA(addr) => self.jpva(addr),
            RNDVB(x, byte) => self.rndvb(x.into(), byte),
            DRWVVN(x, y, nibble) => self.drwvvn(x.into(), y.into(), nibble),
            SKPV(x) => self.skpv(x.into(), keypad),
            SKNPV(x) => self.sknpv(x.into(), keypad),
            LDVDT(x) => self.ldvdt(x.into()),
            LDVK(x) => self.ldvk(x.into(), keypad),
            LDDTV(x) => self.lddtv(x.into()),
            LDSTV(x) => self.ldstv(x.into()),
            ADDIV(x) => self.addiv(x.into()),
            LDFV(x) => self.ldfv(x.into()),
            LDBV(x) => self.ldbv(x.into()),
            LDIV(x) => self.ldiv(x.into()),
            LDVI(x) => self.ldvi(x.into()),
        };

        match result {
            InstructionResult::Nop => (),
            InstructionResult::Next => self.program_counter += INSTRUCTION_SIZE,
            InstructionResult::Skip => self.program_counter += INSTRUCTION_SIZE * 2,
            InstructionResult::Jump(addr) => self.program_counter = addr,
        }
    }

    #[inline]
    fn nop(&self) -> InstructionResult {
        // 0nnn - SYS addr
        // NOP on modern interpreters
        InstructionResult::Nop
    }

    #[inline]
    fn cls(&mut self) -> InstructionResult {
        // 00E0 - CLS
        // Clear the display.
        self.display.clear(false);
        InstructionResult::Next
    }

    #[inline]
    fn ret(&mut self) -> InstructionResult {
        // 00EE - RET
        // Return from a subroutine.
        self.program_counter = self.stack.pop();
        InstructionResult::Next
    }

    #[inline]
    fn jp(&self, addr: u16) -> InstructionResult {
        // 1nnn - JP addr
        // Jump to location nnn.
        InstructionResult::Jump(addr)
    }

    #[inline]
    fn call(&mut self, addr: u16) -> InstructionResult {
        // 2nnn - CALL addr
        // self.cpu.call(arg);
        self.stack.push(self.program_counter);
        InstructionResult::Jump(addr)
    }

    #[inline]
    fn sevb(&self, vx_idx: usize, byte: u8) -> InstructionResult {
        // 3xkk - SE Vx, byte
        // Skip next instruction if Vx = kk.
        if self.registers[vx_idx] == byte {
            InstructionResult::Skip
        } else {
            InstructionResult::Next
        }
    }

    #[inline]
    fn snevb(&self, vx_idx: usize, byte: u8) -> InstructionResult {
        // 4xkk - SNE Vx, byte
        // Skip next instruction if Vx != kk.
        if self.registers[vx_idx] != byte {
            InstructionResult::Skip
        } else {
            InstructionResult::Next
        }
    }

    #[inline]
    fn sevv(&self, vx_idx: usize, vy_idx: usize) -> InstructionResult {
        // 5xy0 - SE Vx, Vy
        // Skip next instruction if Vx = Vy.
        if self.registers[vx_idx] == self.registers[vy_idx] {
            InstructionResult::Skip
        } else {
            InstructionResult::Next
        }
    }

    #[inline]
    fn ldvb(&mut self, vx_idx: usize, byte: u8) -> InstructionResult {
        // 6xkk - LD Vx, byte
        // Set Vx = kk.
        self.registers[vx_idx] = byte;
        InstructionResult::Next
    }

    #[inline]
    fn addvb(&mut self, vx_idx: usize, byte: u8) -> InstructionResult {
        // 7xkk - ADD Vx, byte
        // Set Vx = Vx + kk.
        self.registers[vx_idx] = self.registers[vx_idx].overflowing_add(byte).0;
        InstructionResult::Next
    }

    #[inline]
    fn ldvv(&mut self, vx_idx: usize, vy_idx: usize) -> InstructionResult {
        // 8xy0 - LD Vx, Vy
        // Set Vx = Vy.
        self.registers[vx_idx] = self.registers[vy_idx];
        InstructionResult::Next
    }

    #[inline]
    fn orvv(&mut self, vx_idx: usize, vy_idx: usize) -> InstructionResult {
        // 8xy1 - OR Vx, Vy
        // Set Vx = Vx OR Vy.
        self.registers[vx_idx] |= self.registers[vy_idx];
        InstructionResult::Next
    }

    #[inline]
    fn andvv(&mut self, vx_idx: usize, vy_idx: usize) -> InstructionResult {
        // 8xy2 - AND Vx, Vy
        // Set Vx = Vx AND Vy.
        self.registers[vx_idx] &= self.registers[vy_idx];
        InstructionResult::Next
    }

    #[inline]
    fn xorvv(&mut self, vx_idx: usize, vy_idx: usize) -> InstructionResult {
        // 8xy3 - XOR Vx, Vy
        // Set Vx = Vx XOR Vy.
        self.registers[vx_idx] ^= self.registers[vy_idx];
        InstructionResult::Next
    }

    #[inline]
    fn addvv(&mut self, vx_idx: usize, vy_idx: usize) -> InstructionResult {
        // 8xy4 - ADD Vx, Vy
        // Set Vx = Vx + Vy, set VF = carry.
        let (result, carry) = self.registers[vx_idx].overflowing_add(self.registers[vy_idx]);
        self.registers[vx_idx] = result;
        self.registers[0xF] = carry as u8;
        InstructionResult::Next
    }

    #[inline]
    fn subvv(&mut self, vx_idx: usize, vy_idx: usize) -> InstructionResult {
        // 8xy5 - SUB Vx, Vy
        // Set Vx = Vx - Vy, set VF = NOT borrow.
        let (result, carry) = self.registers[vx_idx].overflowing_sub(self.registers[vy_idx]);
        self.registers[vx_idx] = result;
        self.registers[0xF] = !carry as u8;
        InstructionResult::Next
    }

    #[inline]
    fn shrvv(&mut self, vx_idx: usize, _vy_idx: usize) -> InstructionResult {
        // 8xy6 - SHR Vx {, Vy}
        // Set Vx = Vx SHR 1.

        if self.registers[vx_idx] & 1 != 0 {
            self.registers[0xF] = 1
        } else {
            self.registers[0xF] = 0
        }

        self.registers[vx_idx] /= 2;
        InstructionResult::Next
    }

    #[inline]
    fn subnvv(&mut self, vx_idx: usize, vy_idx: usize) -> InstructionResult {
        // 8xy7 - SUBN Vx, Vy
        // Set Vx = Vy - Vx, set VF = NOT borrow.
        let (result, carry) = self.registers[vy_idx].overflowing_sub(self.registers[vx_idx]);
        self.registers[vx_idx] = result;
        self.registers[0xF] = !carry as u8;
        InstructionResult::Next
    }

    #[inline]
    fn shlvv(&mut self, vx_idx: usize, _vy_idx: usize) -> InstructionResult {
        // 8xyE - SHL Vx {, Vy}
        // Set Vx = Vx SHL 1.

        if self.registers[vx_idx] & (1 << 7) != 0 {
            self.registers[0xF] = 1
        } else {
            self.registers[0xF] = 0
        }

        self.registers[vx_idx] = self.registers[vx_idx].overflowing_mul(2).0;
        InstructionResult::Next
    }

    #[inline]
    fn snevv(&mut self, vx_idx: usize, vy_idx: usize) -> InstructionResult {
        // 9xy0 - SNE Vx, Vy
        // Skip next instruction if Vx != Vy.
        if self.registers[vx_idx] != self.registers[vy_idx] {
            InstructionResult::Skip
        } else {
            InstructionResult::Next
        }
    }

    #[inline]
    fn ldia(&mut self, addr: u16) -> InstructionResult {
        // Annn - LD I, addr
        // self.cpu.ldi(arg);
        self.index_register = addr;
        InstructionResult::Next
    }

    #[inline]
    fn jpva(&self, addr: u16) -> InstructionResult {
        // Bnnn - JP V0, addr
        // Jump to location nnn + V0.
        InstructionResult::Jump(addr + self.registers[0x0] as u16)
    }

    #[inline]
    fn rndvb(&mut self, vx_idx: usize, byte: u8) -> InstructionResult {
        // Cxkk - RND Vx, byte
        // Set Vx = random byte AND kk.
        let num: u8 = self.rng.gen_range(0..255);
        self.registers[vx_idx] = num & byte;
        InstructionResult::Next
    }

    #[inline]
    fn drwvvn(&mut self, vx_idx: usize, vy_idx: usize, nibble: u8) -> InstructionResult {
        self.registers[0xF] = 0;
        let (gfx_width, gfx_height) = self.display.get_current_mode();

        for byte in 0..nibble {
            let y = (self.registers[vy_idx].overflowing_add(byte).0) % gfx_height as u8;
            for bit in 0..8 {
                let x = (self.registers[vx_idx].overflowing_add(bit).0) % gfx_width as u8;
                let color =
                    (self.memory.get8((self.index_register + byte as u16).into()) >> (7 - bit)) & 1;

                let current_pixel_state = self.display.get(x.into(), y.into()) as u8;
                self.registers[0x0f] |= color & current_pixel_state;
                self.display
                    .set(x.into(), y.into(), (current_pixel_state ^ color) != 0);
            }
        }

        InstructionResult::Next
    }

    #[inline]
    fn skpv(&self, vx_idx: usize, keypad: &KeypadState) -> InstructionResult {
        // Ex9E - SKP Vx
        // Skip next instruction if key with the value of Vx is pressed.
        if keypad.state[self.registers[vx_idx] as usize] {
            return InstructionResult::Skip;
        }
        InstructionResult::Next
    }

    #[inline]
    fn sknpv(&self, vx_idx: usize, keypad: &KeypadState) -> InstructionResult {
        // ExA1 - SKNP Vx
        // Skip next instruction if key with the value of Vx is not pressed.
        if !keypad.state[self.registers[vx_idx] as usize] {
            return InstructionResult::Skip;
        }
        InstructionResult::Next
    }

    #[inline]
    fn ldvdt(&mut self, vx_idx: usize) -> InstructionResult {
        // Fx07 - LD Vx, DT
        // Set Vx = delay timer value.
        self.registers[vx_idx] = self.delay_timer;
        InstructionResult::Next
    }

    #[inline]
    fn ldvk(&mut self, vx_idx: usize, keypad: &KeypadState) -> InstructionResult {
        // Fx0A - LD Vx, K
        // Wait for a key press, store the value of the key in Vx.
        for (i, state) in keypad.state.iter().enumerate() {
            if *state {
                self.registers[vx_idx] = i as u8;
                return InstructionResult::Next;
            }
        }
        InstructionResult::Nop
    }

    #[inline]
    fn lddtv(&mut self, vx_idx: usize) -> InstructionResult {
        // Fx15 - LD DT, Vx
        // Set delay timer = Vx.
        self.delay_timer = self.registers[vx_idx];
        InstructionResult::Next
    }

    #[inline]
    fn ldstv(&mut self, vx_idx: usize) -> InstructionResult {
        // Fx18 - LD ST, Vx
        // Set sound timer = Vx.
        self.sound_timer = self.registers[vx_idx];
        InstructionResult::Next
    }

    #[inline]
    fn addiv(&mut self, vx_idx: usize) -> InstructionResult {
        // Fx1E - ADD I, Vx
        // Set I = I + Vx.
        self.index_register += self.registers[vx_idx] as u16;
        InstructionResult::Next
    }

    #[inline]
    fn ldfv(&mut self, vx_idx: usize) -> InstructionResult {
        // Fx29 - LD F, Vx
        // Set I = location of sprite for digit Vx.
        self.index_register = self
            .memory
            .get_font_sprite_location(self.registers[vx_idx].into());
        InstructionResult::Next
    }

    #[inline]
    fn ldbv(&mut self, vx_idx: usize) -> InstructionResult {
        // Fx33 - LD B, Vx
        // Store BCD representation of Vx in memory locations I, I+1, and I+2.
        self.memory
            .set8(self.index_register.into(), self.registers[vx_idx] / 100);
        self.memory.set8(
            (self.index_register + 1).into(),
            (self.registers[vx_idx] / 10) % 10,
        );
        self.memory.set8(
            (self.index_register + 2).into(),
            (self.registers[vx_idx] % 100) % 10,
        );

        InstructionResult::Next
    }

    #[inline]
    fn ldiv(&mut self, vx_idx: usize) -> InstructionResult {
        // Fx55 - LD [I], Vx
        // Store registers V0 through Vx in memory starting at location I.
        for i in 0..=vx_idx {
            self.memory.set8(
                (self.index_register + i as u16).into(),
                self.registers[i as usize],
            )
        }
        InstructionResult::Next
    }

    #[inline]
    fn ldvi(&mut self, vx_idx: usize) -> InstructionResult {
        // Fx65 - LD Vx, [I]
        // Read registers V0 through Vx from memory starting at location I.
        for i in 0..=vx_idx {
            self.registers[i] = self.memory.get8((self.index_register + i as u16).into());
        }
        InstructionResult::Next
    }
}

#[cfg(test)]
mod tests {
    use super::{InstructionResult, INSTRUCTION_SIZE, VM};
    use crate::{keypad::KeypadState, opcode::OpCode};

    // Test helper
    fn execute_opcode(vm: &mut VM, opcode: OpCode) {
        vm.execute(opcode, &KeypadState::new())
    }

    #[test]
    fn nop_test() {
        let vm = VM::new(&[]);
        // Since vm is not mut it can not change
        assert!(vm.nop() == InstructionResult::Next);
    }

    #[test]
    fn cls_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;

        execute_opcode(&mut vm, OpCode::CLS());

        let (gfx_width, gfx_height) = vm.display.get_current_mode();
        let display_mods = vm.display.pop_modifications().expect("No modifications");
        assert_eq!(display_mods.offset, 0);
        assert_eq!(
            display_mods.data.len(),
            gfx_height * (gfx_width / (std::mem::size_of::<u32>() * 8))
        );

        assert_eq!(init_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn call_ret_test() {
        let mut vm = VM::new(&[]);
        let call_addr = 0x0;
        let init_addr = vm.program_counter;
        assert!(call_addr != init_addr);

        execute_opcode(&mut vm, OpCode::CALL(call_addr));
        assert_eq!(call_addr, vm.program_counter);
        execute_opcode(&mut vm, OpCode::RET());
        assert_eq!(init_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn jp_test() {
        let mut vm = VM::new(&[]);
        let jp_addr = 0x0;

        execute_opcode(&mut vm, OpCode::JP(jp_addr));
        assert_eq!(jp_addr, vm.program_counter);
    }

    #[test]
    fn sevb_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        vm.registers[0x0] = 0x10;

        execute_opcode(&mut vm, OpCode::SEVB(0x0, 0x10));
        let next_addr = init_addr + INSTRUCTION_SIZE * 2;
        // Skips
        assert_eq!(next_addr, vm.program_counter);
        execute_opcode(&mut vm, OpCode::SEVB(0x0, 0x11));
        // Does not skip
        assert_eq!(next_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn snevb_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        vm.registers[0x0] = 0x10;

        execute_opcode(&mut vm, OpCode::SNEVB(0x0, 0x10));
        let next_addr = init_addr + INSTRUCTION_SIZE;
        // Does not skip
        assert_eq!(next_addr, vm.program_counter);
        execute_opcode(&mut vm, OpCode::SNEVB(0x0, 0x11));
        // Skips
        assert_eq!(next_addr + INSTRUCTION_SIZE * 2, vm.program_counter);
    }

    #[test]
    fn sevv_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        vm.registers[0x0] = 0x10;
        vm.registers[0x1] = 0x10;

        execute_opcode(&mut vm, OpCode::SEVV(0x0, 0x1));
        let next_addr = init_addr + INSTRUCTION_SIZE * 2;
        // Skips
        assert_eq!(next_addr, vm.program_counter);
        execute_opcode(&mut vm, OpCode::SEVV(0x0, 0x2));
        // Does not skip
        assert_eq!(next_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn ldvb_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        let new_value = 0x11;
        vm.registers[0x0] = 0x10;
        execute_opcode(&mut vm, OpCode::LDVB(0x0, new_value));
        assert_eq!(vm.registers[0x0], new_value);
        assert_eq!(init_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn addvb_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        let add_value = 0x1;
        let init_value = 0x2;
        vm.registers[0x0] = init_value;
        execute_opcode(&mut vm, OpCode::ADDVB(0x0, add_value));
        assert_eq!(vm.registers[0x0], init_value + add_value);
        assert_eq!(init_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn ldvv_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        vm.registers[0x0] = 0x10;
        vm.registers[0x1] = 0x11;
        execute_opcode(&mut vm, OpCode::LDVV(0x0, 0x1));
        assert_eq!(vm.registers[0x0], vm.registers[0x1]);
        assert_eq!(init_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn orvv_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        let init_v0 = 0xA;
        vm.registers[0x0] = init_v0;
        vm.registers[0x1] = 0xB;
        execute_opcode(&mut vm, OpCode::ORVV(0x0, 0x1));
        assert_eq!(vm.registers[0x0], init_v0 | vm.registers[0x1]);
        assert_eq!(init_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn andvv_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        let init_v0 = 0xA;
        vm.registers[0x0] = init_v0;
        vm.registers[0x1] = 0xB;
        execute_opcode(&mut vm, OpCode::ANDVV(0x0, 0x1));
        assert_eq!(vm.registers[0x0], init_v0 & vm.registers[0x1]);
        assert_eq!(init_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn xorvv_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        let init_v0 = 0xA;
        vm.registers[0x0] = init_v0;
        vm.registers[0x1] = 0xB;
        execute_opcode(&mut vm, OpCode::XORVV(0x0, 0x1));
        assert_eq!(vm.registers[0x0], init_v0 ^ vm.registers[0x1]);
        assert_eq!(init_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn addvv_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        vm.registers[0x0] = 0xA;
        vm.registers[0x1] = 0xB;

        let (result, carry) = vm.registers[0x0].overflowing_add(vm.registers[0x1]);

        execute_opcode(&mut vm, OpCode::ADDVV(0x0, 0x1));
        assert_eq!(vm.registers[0x0], result as u8);
        assert_eq!(vm.registers[0xF], carry as u8);
        assert_eq!(init_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn subvv_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        vm.registers[0x0] = 0xA;
        vm.registers[0x1] = 0xB;

        let (result, carry) = vm.registers[0x0].overflowing_sub(vm.registers[0x1]);

        execute_opcode(&mut vm, OpCode::SUBVV(0x0, 0x1));
        assert_eq!(vm.registers[0x0], result as u8);
        assert_eq!(vm.registers[0xF], !carry as u8);
        assert_eq!(init_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn shrvv_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        let init_value = 0xA;
        vm.registers[0x2] = init_value;

        execute_opcode(&mut vm, OpCode::SHRVV(0x2, 0x0));
        assert_eq!(vm.registers[0x2], init_value >> 1);
        assert_eq!(vm.registers[0xF], 0);
        assert_eq!(init_addr + INSTRUCTION_SIZE, vm.program_counter);

        let init_value = 0x1;
        vm.registers[0x2] = init_value;

        execute_opcode(&mut vm, OpCode::SHRVV(0x2, 0x0));
        assert_eq!(vm.registers[0x2], init_value >> 1);
        assert_eq!(vm.registers[0xF], 1);
    }

    #[test]
    fn subnvv_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        vm.registers[0x0] = 0xA;
        vm.registers[0x1] = 0xB;

        let (result, carry) = vm.registers[0x1].overflowing_sub(vm.registers[0x0]);

        execute_opcode(&mut vm, OpCode::SUBNVV(0x0, 0x1));
        assert_eq!(vm.registers[0x0], result as u8);
        assert_eq!(vm.registers[0xF], !carry as u8);
        assert_eq!(init_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn shlvv_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        let init_value = 0xA;
        vm.registers[0x2] = init_value;

        execute_opcode(&mut vm, OpCode::SHLVV(0x2, 0x0));
        assert_eq!(vm.registers[0x2], init_value << 1);
        assert_eq!(vm.registers[0xF], 0);
        assert_eq!(init_addr + INSTRUCTION_SIZE, vm.program_counter);

        let init_value = u8::MAX;
        vm.registers[0x2] = init_value;

        execute_opcode(&mut vm, OpCode::SHLVV(0x2, 0x0));
        assert_eq!(vm.registers[0x2], init_value << 1);
        assert_eq!(vm.registers[0xF], 1);
    }

    #[test]
    fn snevv_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        vm.registers[0x0] = 0xA;
        vm.registers[0x1] = 0xA;

        execute_opcode(&mut vm, OpCode::SNEVV(0x0, 0x2));
        let next_addr = init_addr + INSTRUCTION_SIZE * 2;
        // Skips
        assert_eq!(next_addr, vm.program_counter);
        execute_opcode(&mut vm, OpCode::SNEVV(0x0, 0x1));
        // Does not skip
        assert_eq!(next_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn ldia_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        let new_value = 0x11;
        vm.index_register = 0x10;
        execute_opcode(&mut vm, OpCode::LDIA(new_value));
        assert_eq!(vm.index_register, new_value);
        assert_eq!(init_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn jpva_test() {
        let mut vm = VM::new(&[]);
        vm.registers[0x0] = 0xA;
        let jp_addr = 0x1;

        execute_opcode(&mut vm, OpCode::JPVA(jp_addr));
        assert_eq!(jp_addr + vm.registers[0x0] as u16, vm.program_counter);
    }

    #[test]
    fn rndvb_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        let init_val = 0xF;
        vm.registers[0x0] = init_val;

        execute_opcode(&mut vm, OpCode::RNDVB(0x0, 0xA));
        assert_ne!(vm.registers[0x0], init_val);
        assert_eq!(init_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn drwvvn_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        // Flush initial clear()
        vm.display.pop_modifications();

        execute_opcode(&mut vm, OpCode::DRWVVN(0x0, 0x0, 0x1));
        let display_mods = vm.display.pop_modifications().expect("No modifications");
        assert_eq!(display_mods.offset, 0);
        assert_eq!(display_mods.data.len(), 1);
        assert_eq!(init_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn skpv_test() {
        let mut keypad_state = KeypadState::new();
        keypad_state.state[0xA] = true;

        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        vm.registers[0x0] = 0xA;

        vm.execute(OpCode::SKPV(0x0), &keypad_state);
        // Skips
        let next_addr = init_addr + INSTRUCTION_SIZE * 2;
        assert_eq!(next_addr, vm.program_counter);
        vm.execute(OpCode::SKPV(0x1), &keypad_state);
        // Does not skip
        assert_eq!(next_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn sknpv_test() {
        let mut keypad_state = KeypadState::new();
        keypad_state.state[0xA] = true;

        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        vm.registers[0x0] = 0xA;

        vm.execute(OpCode::SKNPV(0x1), &keypad_state);
        // Skips
        let next_addr = init_addr + INSTRUCTION_SIZE * 2;
        assert_eq!(next_addr, vm.program_counter);
        vm.execute(OpCode::SKNPV(0x0), &keypad_state);
        // Does not skip
        assert_eq!(next_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn lddtv_ldvdt_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        let timer_val = 0xA;

        vm.registers[0x2] = timer_val;

        execute_opcode(&mut vm, OpCode::LDDTV(0x2));
        assert_eq!(vm.delay_timer, timer_val);
        assert_eq!(init_addr + INSTRUCTION_SIZE, vm.program_counter);

        execute_opcode(&mut vm, OpCode::LDVDT(0x3));
        assert_eq!(vm.registers[0x3], timer_val);
        assert_eq!(init_addr + INSTRUCTION_SIZE * 2, vm.program_counter);
    }

    #[test]
    fn ldvk_test() {
        let mut keypad_state = KeypadState::new();
        keypad_state.state[0xA] = true;

        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        vm.registers[0x0] = 0xA;

        vm.execute(OpCode::SKNPV(0x1), &keypad_state);
        // Skips
        let next_addr = init_addr + INSTRUCTION_SIZE * 2;
        assert_eq!(next_addr, vm.program_counter);
        vm.execute(OpCode::SKNPV(0x0), &keypad_state);
        // Does not skip
        assert_eq!(next_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn ldstv_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        let timer_val = 0xA;

        vm.registers[0x2] = timer_val;

        execute_opcode(&mut vm, OpCode::LDSTV(0x2));
        assert_eq!(vm.sound_timer, timer_val);
        assert_eq!(init_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn addiv_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        let start_value = 0x1;
        let add_value = 0x3;
        vm.registers[0xA] = add_value;
        vm.index_register = start_value;
        execute_opcode(&mut vm, OpCode::ADDIV(0xA));
        assert_eq!(vm.index_register, start_value + add_value as u16);
        assert_eq!(init_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn ldfv_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        vm.registers[0xA] = 0x3;

        execute_opcode(&mut vm, OpCode::LDFV(0xA));
        assert_eq!(vm.index_register, 0x050 + (5 * vm.registers[0xA]) as u16);
        assert_eq!(init_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn ldbv_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;

        vm.registers[0xA] = 0x10;
        vm.index_register = 0xAA;

        execute_opcode(&mut vm, OpCode::LDBV(0xA));
        assert_eq!(
            vm.memory.get8(vm.index_register as usize),
            vm.registers[0xA] / 100
        );
        assert_eq!(
            vm.memory.get8((vm.index_register + 1) as usize),
            (vm.registers[0xA] / 10) % 10
        );
        assert_eq!(
            vm.memory.get8((vm.index_register + 2) as usize),
            (vm.registers[0xA] % 100) % 10
        );
        assert_eq!(init_addr + INSTRUCTION_SIZE, vm.program_counter);
    }

    #[test]
    fn ldiv_ldvi_test() {
        let mut vm = VM::new(&[]);
        let init_addr = vm.program_counter;
        let max_reg = 0x4 as u8;
        vm.index_register = 0xAA;

        for i in 0x0..=max_reg {
            vm.registers[0x0 + i as usize] = i as u8 + 0x10;
        }

        execute_opcode(&mut vm, OpCode::LDIV(max_reg));
        for i in 0x0..=max_reg {
            assert_eq!(
                vm.memory.get8((vm.index_register + i as u16) as usize),
                vm.registers[0x0 + i as usize]
            );
        }
        assert_eq!(init_addr + INSTRUCTION_SIZE, vm.program_counter);

        for i in 0x0..=max_reg {
            vm.registers[0x0 + i as usize] = 0x0;
        }

        execute_opcode(&mut vm, OpCode::LDVI(max_reg));
        for i in 0x0..=max_reg {
            assert_eq!(
                vm.memory.get8((vm.index_register + i as u16) as usize),
                vm.registers[0x0 + i as usize]
            );
        }
        assert_eq!(init_addr + INSTRUCTION_SIZE * 2, vm.program_counter);
    }
}
