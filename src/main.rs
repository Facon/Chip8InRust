use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

const V_SIZE: usize = 16;
const STACK_SIZE: usize = 16;
const MEMORY_SIZE: usize = 4096;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const KEYBOARD_SIZE: usize = 16;

#[derive(Debug)]
struct Chip8State {
    v: [u8; V_SIZE],
    i: u16,
    delay_timer: u8,
    sound_timer: u8,
    pc: u16,
    sp: u8,
    stack: Vec<u16>,
    memory: Vec<u8>
}

impl Chip8State {
    fn new() -> Chip8State {
        Chip8State { 
            v: [0; V_SIZE],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            pc: 0,
            sp: 0, 
            stack: vec![0; STACK_SIZE], 
            memory: vec![0; MEMORY_SIZE]
        }
    }

    fn with_all_instructions() -> Chip8State {
        let mut chip8 = Chip8State::new();

        chip8.memory[0] = 0x00; chip8.memory[1] = 0xE0;
        chip8.memory[2] = 0x00; chip8.memory[3] = 0xEE;
        chip8.memory[4] = 0x01; chip8.memory[5] = 0x11;
        chip8.memory[6] = 0x12; chip8.memory[7] = 0x22;
        chip8.memory[8] = 0x23; chip8.memory[9] = 0x33;
        chip8.memory[10] = 0x3E; chip8.memory[11] = 0xAA;
        chip8.memory[12] = 0x4F; chip8.memory[13] = 0xBB;
        chip8.memory[14] = 0x51; chip8.memory[15] = 0x20;
        chip8.memory[16] = 0x6F; chip8.memory[17] = 0x10;
        chip8.memory[18] = 0x70; chip8.memory[19] = 0x01;
        chip8.memory[20] = 0x81; chip8.memory[21] = 0x10;
        chip8.memory[22] = 0x82; chip8.memory[23] = 0x21;
        chip8.memory[24] = 0x83; chip8.memory[25] = 0x32;
        chip8.memory[26] = 0x84; chip8.memory[27] = 0x43;
        chip8.memory[28] = 0x85; chip8.memory[29] = 0x54;
        chip8.memory[30] = 0x86; chip8.memory[31] = 0x65;
        chip8.memory[32] = 0x87; chip8.memory[33] = 0x76;
        chip8.memory[34] = 0x88; chip8.memory[35] = 0x87;
        chip8.memory[36] = 0x89; chip8.memory[37] = 0x9E;
        chip8.memory[38] = 0x98; chip8.memory[39] = 0x80;
        chip8.memory[40] = 0xA9; chip8.memory[41] = 0x99;
        chip8.memory[42] = 0xBA; chip8.memory[43] = 0xAA;
        chip8.memory[44] = 0xCF; chip8.memory[45] = 0xBB;
        chip8.memory[46] = 0xD1; chip8.memory[47] = 0x23;
        chip8.memory[48] = 0xE0; chip8.memory[49] = 0x9E;
        chip8.memory[50] = 0xE0; chip8.memory[51] = 0xA1;
        chip8.memory[52] = 0xF1; chip8.memory[53] = 0x07;
        chip8.memory[54] = 0xF2; chip8.memory[55] = 0x0A;
        chip8.memory[56] = 0xF3; chip8.memory[57] = 0x15;
        chip8.memory[58] = 0xF4; chip8.memory[59] = 0x18;
        chip8.memory[60] = 0xF5; chip8.memory[61] = 0x1E;
        chip8.memory[62] = 0xF6; chip8.memory[63] = 0x29;
        chip8.memory[64] = 0xF7; chip8.memory[65] = 0x33;
        chip8.memory[66] = 0xF8; chip8.memory[67] = 0x55;
        chip8.memory[68] = 0xF9; chip8.memory[69] = 0x65;

        chip8
    }
}

struct Chip8MachineState {
    cycles: u64,
    display: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT],
    keyboard: [bool; KEYBOARD_SIZE],
    state: Chip8State,
	random: ChaCha8Rng
}

impl Chip8MachineState {
    fn new() -> Chip8MachineState {
        Chip8MachineState {
            cycles:0,
            display: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
            keyboard: [false; KEYBOARD_SIZE],
            state: Chip8State::new(),
			random: ChaCha8Rng::from_entropy()
        }
    }

    fn execute_cycle(&mut self) -> ()
    {
        self.cycles += 1;

        let address : usize = self.state.pc as usize;
        let instruction = &self.state.memory[address..(address+2)%self.state.memory.len()];

        let position3 = instruction[0] >> 4;
        let position2 = instruction[0] & 0x0F;
        let position1 = instruction[1] >> 4;
        let position0 = instruction[1] & 0x0F;
        let address: u16 = ((instruction[0] as u16 & 0x0F) << 8) | (instruction[1] as u16);
        let nibble = position0 as usize;
        let x = position2 as usize;
        let y = position1 as usize;
        let byte = instruction[1];

        self.state.pc += 2;

        match (position3, position2, position1, position0) {
            (0x0, 0x0, 0xE, 0x0) => self.execute_cls(),
            (0x0, 0x0, 0xE, 0xE) => self.execute_ret(),
            (0x0, _, _, _) => self.execute_sys_addr(address),
            (0x1, _, _, _) => self.execute_jp_addr(address),
            (0x2, _, _, _) => self.execute_call_addr(address),
            (0x3, _, _, _) => self.execute_se_vx_byte(x, byte),
            (0x4, _, _, _) => self.execute_sne_vx_byte(x, byte),
            (0x5, _, _, 0x0) => self.execute_se_vx_vy(x, y),
            (0x6, _, _, _) => self.execute_ld_vx_byte(x, byte),
            (0x7, _, _, _) => self.execute_add_vx_byte(x, byte),
            (0x8, _, _, 0x0) => self.execute_ld_vx_vy(x, y),
            (0x8, _, _, 0x1) => self.execute_or_vx_vy(x, y),
            (0x8, _, _, 0x2) => self.execute_and_vx_vy(x, y),
            (0x8, _, _, 0x3) => self.execute_xor_vx_vy(x, y),
            (0x8, _, _, 0x4) => self.execute_add_vx_vy(x, y),
            (0x8, _, _, 0x5) => self.execute_sub_vx_vy(x, y),
            (0x8, _, _, 0x6) => self.execute_shr_vx(x),
            (0x8, _, _, 0x7) => self.execute_subn_vx_vy(x, y),
            (0x8, _, _, 0xE) => self.execute_shl_vx(x),
            (0x9, _, _, 0x0) => self.execute_sne_vx_vy(x, y),
            (0xA, _, _, _) => self.execute_ld_i_addr(address),
            (0xD, _, _, _) => self.execute_draw_vx_vy_nibble(x, y, nibble),
            (0xB, _, _, _) => self.execute_jp_v0_addr(address),
            (0xC, _, _, _) => self.execute_rnd_vx_byte(x, byte),
            (0xE, _, 0x9, 0xE) => self.execute_skp_vx(x),
            (0xE, _, 0xA, 0x1) => self.execute_sknp_vx(x),
            (0xF, _, 0x0, 0x7) => self.execute_ld_vx_dt(x),
            (0xF, _, 0x0, 0xA) => self.execute_ld_vx_k(x),
            (0xF, _, 0x1, 0x5) => self.execute_ld_dt_vx(x),
            (0xF, _, 0x1, 0x8) => self.execute_ld_st_vx(x),
            (0xF, _, 0x1, 0xE) => self.execute_add_i_vx(x),
            (0xF, _, 0x2, 0x9) => self.execute_ld_f_vx(x),
            (0xF, _, 0x3, 0x3) => self.execute_ld_b_vx(x),
            (0xF, _, 0x5, 0x5) => self.execute_ld_ref_i_vx(),
            (0xF, _, 0x6, 0x5) => self.execute_ld_vx_ref_i(),
            _ => panic!("Unknown instruction!")
        }
    }

    fn execute_cls(&mut self) {
        for y in self.display.iter_mut() {
            for x in y.iter_mut() {
                *x = false;
            }
        }
    }

    fn execute_ret(&mut self) {
        self.state.sp -= 1;
        self.state.pc = self.state.stack[self.state.sp as usize]
    }

    // To check differences with CALL NNN - 0x2NNN.
    fn execute_sys_addr(&mut self, address: u16) {
        self.execute_call_addr(address);
    }

    fn execute_jp_addr(&mut self, address: u16) {
        self.state.pc = address;
    }

    fn execute_call_addr(&mut self, address: u16) {
        self.state.sp += 1;
        self.state.stack[self.state.sp as usize] = self.state.pc;
        self.state.pc = address;
    }

    fn execute_se_vx_byte(&mut self, x: usize, byte: u8) {
        if self.state.v[x] == byte {
            self.state.pc += 2;
        }
    }

    fn execute_sne_vx_byte(&mut self, x: usize, byte: u8) {
        if self.state.v[x] != byte {
            self.state.pc += 2;
        }
    }

    fn execute_se_vx_vy(&mut self, x: usize, y: usize) {
        if self.state.v[x] == self.state.v[y] {
            self.state.pc += 2;
        }
    }

    fn execute_ld_vx_byte(&mut self, x: usize, byte: u8) {
        self.state.v[x] = byte;
    }

    fn execute_add_vx_byte(&mut self, x: usize, byte: u8) {
        self.state.v[x] += byte;
    }

    fn execute_ld_vx_vy(&mut self, x: usize, y: usize) {
        self.state.v[x] = self.state.v[y];
    }

    fn execute_or_vx_vy(&mut self, x: usize, y: usize) {
        self.state.v[x] |= self.state.v[y];
    }

    fn execute_and_vx_vy(&mut self, x: usize, y: usize) {
        self.state.v[x] &= self.state.v[y];
    }

    fn execute_xor_vx_vy(&mut self, x: usize, y: usize) {
        self.state.v[x] ^= self.state.v[y];
    }

    fn execute_add_vx_vy(&mut self, x: usize, y: usize) {
        let (result, overflow) = self.state.v[x].overflowing_add(self.state.v[y]);
        self.state.v[x] = result;
        self.state.v[0xF] = overflow as u8;
    }

    fn execute_sub_vx_vy(&mut self, x: usize, y: usize) {
        let (result, overflow) = self.state.v[x].overflowing_sub(self.state.v[y]);
        self.state.v[x] = result;
        self.state.v[0xF] = (!overflow) as u8; // NOT borrow!
    }

    fn execute_shr_vx(&mut self, x: usize) {
        self.state.v[0xF] = self.state.v[x] & 0x1;
        self.state.v[x] >>= 1;
    }

    fn execute_subn_vx_vy(&mut self, x: usize, y: usize) {
        let (result, overflow) = self.state.v[y].overflowing_sub(self.state.v[x]);
        self.state.v[x] = result;
        self.state.v[0xF] = (!overflow) as u8; // NOT borrow!
    }

    fn execute_shl_vx(&mut self, x: usize) {
        self.state.v[0xF] = (self.state.v[x] & 0x80) >> 7;
        self.state.v[x] <<= 1;
    }

    fn execute_sne_vx_vy(&mut self, x: usize, y: usize) {
        self.state.pc += if self.state.v[x] != self.state.v[y] { 2 } else { 0 };
    }

    fn execute_ld_i_addr(&mut self, address: u16) {
        self.state.i = address;
    }

    fn execute_jp_v0_addr(&mut self, address: u16) {
        self.state.pc = address + self.state.v[0] as u16;
    }

    fn execute_rnd_vx_byte(&mut self, x: usize, byte: u8) {
        self.state.v[x] = self.random.gen::<u8>() & byte;
    }

    fn execute_draw_vx_vy_nibble(&mut self, x: usize, y: usize, nibble: usize) {
        let mut collision = false;
        let memory_start_position = self.state.i as usize;
        let sprite = &self.state.memory[memory_start_position .. memory_start_position + nibble];
        let copied_sprite = sprite.to_vec();
        let rows = copied_sprite.len();

        for j in 0..rows {
            let row = copied_sprite[j];

            for i in 0..8 {
                let new_value = row >> (7 - i) & 0x01;

                if new_value == 1 {
                    let xi = (self.state.v[x] as usize + i) % SCREEN_WIDTH;
                    let yj = (self.state.v[y] as usize + j) % SCREEN_HEIGHT;
                    let old_value = self.get_pixel(xi, yj);
                    
                    if old_value {
                        collision = true;
                    }

                    self.set_pixel(xi, yj, (new_value == 1) ^ old_value);
                }
            }
        }

        self.state.v[0xF] = collision as u8;
    }

    fn set_pixel(&mut self, x: usize, y: usize, on: bool) {
        self.display[y][x] = on;
    }
    
    fn get_pixel(&self, x: usize, y: usize) -> bool {
        self.display[y][x]
    }

    fn execute_skp_vx(&mut self, x: usize) {
        self.state.pc += if self.keyboard[self.state.v[x] as usize] { 2 } else { 0 };
    }

    fn execute_sknp_vx(&mut self, x: usize) {
        self.state.pc += if self.keyboard[self.state.v[x] as usize] { 0 } else { 2 };
    }

    fn execute_ld_vx_dt(&mut self, x: usize) {
        self.state.v[x] = self.state.delay_timer;
    }

    fn execute_ld_vx_k(&mut self, x: usize) {
        self.state.pc -= 2;
        let key_press = self.keyboard.iter().position(|&x| x == true);
        
        if key_press.is_some() {
            self.state.v[x] = key_press.unwrap() as u8;
            self.state.pc += 2;
        }
    }

    fn execute_ld_dt_vx(&mut self, x: usize) {
        self.state.delay_timer = self.state.v[x];
    }

    fn execute_ld_st_vx(&mut self, x: usize) {
        self.state.sound_timer = self.state.v[x];
    }

    fn execute_add_i_vx(&mut self, x: usize) {
        self.state.i += self.state.v[x] as u16;
    }

    fn execute_ld_f_vx(&mut self, x: usize) {
        self.state.i = self.state.v[x] as u16 * 5;
    }

    fn execute_ld_b_vx(&mut self, x: usize) {
        let index : usize = self.state.i.into();

        self.state.memory[index] = self.state.v[x] / 100;
        self.state.memory[index + 1] = (self.state.v[x] / 10) % 10;
        self.state.memory[index + 2] = (self.state.v[x] % 100) % 10;
    }

    fn execute_ld_ref_i_vx(&mut self) {
        for i in 0..self.state.v.len() {
            self.state.memory[self.state.i as usize + i] = self.state.v[i];
        }
    }

    fn execute_ld_vx_ref_i(&mut self) {
        for i in 0..self.state.v.len() {
            self.state.v[i] = self.state.memory[self.state.i as usize + i];
        }
    }
}

trait DisassemblyOutput {
    fn disassemble(&self, address: usize) -> String;
}

fn disassemble(cpu: &impl DisassemblyOutput, address: usize) -> String {
	return cpu.disassemble(address);
}

impl DisassemblyOutput for Chip8State
{
    fn disassemble(&self, address: usize) -> String {
        let instruction = &self.memory[address..(address+2)%self.memory.len()];
        format!("{:#04X}: {:02X}{:02X} -> {}", address, instruction[0], instruction[1], 
            decrypt_chip8_instruction(instruction))
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

fn main() {
    println!("Hello, world!");

    let x = 20;
    let y = 10;

    println!("{}", x + y);
}

#[test]
fn should_create() {
    let mut cpu = Chip8State::new();

    cpu.memory[5] = 3;

    println!("{:?}", cpu);
}

#[test]
fn should_print_instruction() {
    let cpu = Chip8State::new();

    println!("{}", disassemble(&cpu, 0));
}

#[test]
fn should_print_all_instructions() {
    let cpu = Chip8State::with_all_instructions();

    for address in 0..35 {
        println!("{}", disassemble(&cpu, address*2));
    }
}