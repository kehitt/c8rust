pub enum OpCode {
    NOP(),
    CLS(),
    RET(),
    JP(u16),
    CALL(u16),
    SEVB(u8, u8),
    SNEVB(u8, u8),
    SEVV(u8, u8),
    LDVB(u8, u8),
    ADDVB(u8, u8),
    LDVV(u8, u8),
    ORVV(u8, u8),
    ANDVV(u8, u8),
    XORVV(u8, u8),
    ADDVV(u8, u8),
    SUBVV(u8, u8),
    SHRVV(u8, u8),
    SUBNVV(u8, u8),
    SHLVV(u8, u8),
    SNEVV(u8, u8),
    LDIA(u16),
    JPVA(u16),
    RNDVB(u8, u8),
    DRWVVN(u8, u8, u8),
    SKPV(u8),
    SKNPV(u8),
    LDVDT(u8),
    LDVK(u8),
    LDDTV(u8),
    LDSTV(u8),
    ADDIV(u8),
    LDFV(u8),
    LDBV(u8),
    LDIV(u8),
    LDVI(u8),
}

impl OpCode {
    pub fn from_bytes(bytes: u16) -> Self {
        use OpCode::*;

        match Self::split_bytes(bytes) {
            (0x0, 0x0, 0x0, 0x0) => NOP(),
            (0x0, 0x0, 0xE, 0x0) => CLS(),
            (0x0, 0x0, 0xE, 0xE) => RET(),
            (0x1, _, _, _) => JP(Self::get_addr(bytes)),
            (0x2, _, _, _) => CALL(Self::get_addr(bytes)),
            (0x3, x, _, _) => SEVB(x, Self::get_byte(bytes)),
            (0x4, x, _, _) => SNEVB(x, Self::get_byte(bytes)),
            (0x5, x, y, 0x0) => SEVV(x, y),
            (0x6, x, _, _) => LDVB(x, Self::get_byte(bytes)),
            (0x7, x, _, _) => ADDVB(x, Self::get_byte(bytes)),
            (0x8, x, y, 0x0) => LDVV(x, y),
            (0x8, x, y, 0x1) => ORVV(x, y),
            (0x8, x, y, 0x2) => ANDVV(x, y),
            (0x8, x, y, 0x3) => XORVV(x, y),
            (0x8, x, y, 0x4) => ADDVV(x, y),
            (0x8, x, y, 0x5) => SUBVV(x, y),
            (0x8, x, y, 0x6) => SHRVV(x, y),
            (0x8, x, y, 0x7) => SUBNVV(x, y),
            (0x8, x, y, 0xE) => SHLVV(x, y),
            (0x9, x, y, 0x0) => SNEVV(x, y),
            (0xA, _, _, _) => LDIA(Self::get_addr(bytes)),
            (0xB, _, _, _) => JPVA(Self::get_addr(bytes)),
            (0xC, x, _, _) => RNDVB(x, Self::get_byte(bytes)),
            (0xD, x, y, n) => DRWVVN(x, y, n),
            (0xE, x, 0x9, 0xE) => SKPV(x),
            (0xE, x, 0xA, 0x1) => SKNPV(x),
            (0xF, x, 0x0, 0x7) => LDVDT(x),
            (0xF, x, 0x0, 0xA) => LDVK(x),
            (0xF, x, 0x1, 0x5) => LDDTV(x),
            (0xF, x, 0x1, 0x8) => LDSTV(x),
            (0xF, x, 0x1, 0xE) => ADDIV(x),
            (0xF, x, 0x2, 0x9) => LDFV(x),
            (0xF, x, 0x3, 0x3) => LDBV(x),
            (0xF, x, 0x5, 0x5) => LDIV(x),
            (0xF, x, 0x6, 0x5) => LDVI(x),
            _ => panic!("Unknown opcode: {:#04x}", bytes),
        }
    }

    #[inline]
    fn split_bytes(bytes: u16) -> (u8, u8, u8, u8) {
        (
            ((bytes & 0xF000) >> 12) as u8,
            ((bytes & 0x0F00) >> 8) as u8,
            ((bytes & 0x00F0) >> 4) as u8,
            (bytes & 0x000F) as u8,
        )
    }

    /*
        As per (http://devernay.free.fr/hacks/chip8/C8TECH10.HTM):
        '''
            nnn or addr - A 12-bit value, the lowest 12 bits of the instruction
            n or nibble - A 4-bit value, the lowest 4 bits of the instruction
            x - A 4-bit value, the lower 4 bits of the high byte of the instruction
            y - A 4-bit value, the upper 4 bits of the low byte of the instruction
            kk or byte - An 8-bit value, the lowest 8 bits of the instruction
        '''

        This codebase refers to the last 12 bits of an opcode
        as "addr", and to the last 8 bits of an opcode as "byte"
    */

    #[inline]
    fn get_addr(bytes: u16) -> u16 {
        bytes & 0x0FFF
    }

    #[inline]
    fn get_byte(bytes: u16) -> u8 {
        (bytes & 0x00FF) as u8
    }
}
