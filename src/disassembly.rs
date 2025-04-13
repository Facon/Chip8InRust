use crate::chip8_state::Chip8State;

pub trait DisassemblyOutput {
    fn disassemble(&self, address: usize) -> String;
}

impl DisassemblyOutput for Chip8State {
    fn disassemble(&self, address: usize) -> String {
        let instruction = &self.memory[address..address + 2];
        format!(
            "{:#04X}: {:02X}{:02X} -> {}",
            address,
            instruction[0],
            instruction[1],
            decrypt_chip8_instruction(instruction)
        )
    }
}

fn decrypt_chip8_instruction(instruction: &[u8]) -> String {
    let position3 = instruction[0] >> 4;
    let position2 = instruction[0] & 0x0F;
    let position1 = instruction[1] >> 4;
    let position0 = instruction[1] & 0x0F;
    let address: u16 = ((instruction[0] as u16 & 0x0F) << 8) | (instruction[1] as u16);
    let nibble = position0;
    let x = position2;
    let y = position1;
    let byte = instruction[1];

    match (position3, position2, position1, position0) {
        (0x0, 0x0, 0xE, 0x0) => "CLS".to_string(),
        (0x0, 0x0, 0xE, 0xE) => "RET".to_string(),
        (0x0, _, _, _) => format!("SYS {:#05X}", address),
        (0x1, _, _, _) => format!("JP {:#05X}", address),
        (0x2, _, _, _) => format!("CALL {:#05X}", address),
        (0x3, _, _, _) => format!("SE V{:X}, {:#04X}", x, byte),
        (0x4, _, _, _) => format!("SNE V{:X}, {:#04X}", x, byte),
        (0x5, _, _, 0x0) => format!("SE V{:X}, V{:X}", x, y),
        (0x6, _, _, _) => format!("LD V{:X}, {:#04X}", x, byte),
        (0x7, _, _, _) => format!("ADD V{:X}, {:#04X}", x, byte),
        (0x8, _, _, 0x0) => format!("LD V{:X}, V{:X}", x, y),
        (0x8, _, _, 0x1) => format!("OR V{:X}, V{:X}", x, y),
        (0x8, _, _, 0x2) => format!("AND V{:X}, V{:X}", x, y),
        (0x8, _, _, 0x3) => format!("XOR V{:X}, V{:X}", x, y),
        (0x8, _, _, 0x4) => format!("ADD V{:X}, V{:X}", x, y),
        (0x8, _, _, 0x5) => format!("SUB V{:X}, V{:X}", x, y),
        (0x8, _, _, 0x6) => format!("SHR V{:X}", x),
        (0x8, _, _, 0x7) => format!("SUBN V{:X}, V{:X}", x, y),
        (0x8, _, _, 0xE) => format!("SHL V{:X}", x),
        (0x9, _, _, 0x0) => format!("SNE V{:X}, V{:X}", x, y),
        (0xA, _, _, _) => format!("LD I, {:#05X}", address),
        (0xB, _, _, _) => format!("JP V0, {:#05X}", address),
        (0xC, _, _, _) => format!("RND V{:X}, {:X}", x, byte),
        (0xD, _, _, _) => format!("DRW V{:X}, V{:X}, {:#X}", x, y, nibble),
        (0xE, _, 0x9, 0xE) => format!("SKP V{:X}", x),
        (0xE, _, 0xA, 0x1) => format!("SKNP V{:X}", x),
        (0xF, _, 0x0, 0x7) => format!("LD V{:X}, DT", x),
        (0xF, _, 0x0, 0xA) => format!("LD V{:X}, K", x),
        (0xF, _, 0x1, 0x5) => format!("LD DT, V{:X}", x),
        (0xF, _, 0x1, 0x8) => format!("LD ST, V{:X}", x),
        (0xF, _, 0x1, 0xE) => format!("ADD I, V{:X}", x),
        (0xF, _, 0x2, 0x9) => format!("LD F, V{:X}", x),
        (0xF, _, 0x3, 0x3) => format!("LD B, V{:X}", x),
        (0xF, _, 0x5, 0x5) => format!("LD [I], V{:X}", x),
        (0xF, _, 0x6, 0x5) => format!("LD V{:X}, [I]", x),
        _ => "Unknown instruction".to_string()
    }
}