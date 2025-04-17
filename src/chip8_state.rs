use std::{fs::File, io::Read, path::Path};

pub const V_SIZE: usize = 16;
pub const STACK_SIZE: usize = 16;
pub const MEMORY_SIZE: usize = 4096;

#[derive(Debug)]
pub struct Chip8State {
    pub v: [u8; V_SIZE],
    pub i: u16,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub pc: u16,
    pub sp: u8,
    pub stack: [u16; STACK_SIZE],
    pub memory: [u8; MEMORY_SIZE],
}

impl Chip8State {
    pub fn new() -> Self {
        Self {
            v: [0; V_SIZE],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            pc: 0x200, // Program counter starts at 0x200
            sp: 0,
            stack: [0; STACK_SIZE],
            memory: [0; MEMORY_SIZE],
        }
    }

    pub fn with_all_instructions() -> Chip8State {
        let mut chip8 = Chip8State::new();

        // Initialize memory with instructions...
        let instructions = [
            0x00, 0xE0, 0x00, 0xEE, 0x01, 0x11, 0x12, 0x22, 0x23, 0x33, 0x3E, 0xAA, 0x4F, 0xBB,
            0x51, 0x20, 0x6F, 0x10, 0x70, 0x01, 0x81, 0x10, 0x82, 0x21, 0x83, 0x32, 0x84, 0x43,
            0x85, 0x54, 0x86, 0x65, 0x87, 0x76, 0x88, 0x87, 0x89, 0x9E, 0x98, 0x80, 0xA9, 0x99,
            0xBA, 0xAA, 0xCF, 0xBB, 0xD1, 0x23, 0xE0, 0x9E, 0xE0, 0xA1, 0xF1, 0x07, 0xF2, 0x0A,
            0xF3, 0x15, 0xF4, 0x18, 0xF5, 0x1E, 0xF6, 0x29, 0xF7, 0x33, 0xF8, 0x55, 0xF9, 0x65,
            0xFA, 0x81, // Unknown instruction
        ];

        for (i, &byte) in instructions.iter().enumerate() {
            chip8.memory[0x200 + i] = byte;
        }

        chip8
    }

    pub fn load_rom<P: AsRef<Path>>(&mut self, rom_path: P, address: usize) -> std::io::Result<()> {
        let mut file = File::open(rom_path)?;
        let mut rom_buffer = Vec::new();
        file.read_to_end(&mut rom_buffer)?;

        let end_address = address + rom_buffer.len();

        if end_address >= MEMORY_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "ROM file too large for CHIP-8 memory",
            ));
        }

        self.memory[address..end_address].copy_from_slice(&rom_buffer);
        Ok(())
    }
}
