use crate::chip8_state::Chip8State;
use rand_chacha::ChaCha8Rng;
use rand::prelude::*;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const KEYBOARD_SIZE: usize = 16;

pub struct Chip8MachineState {
    pub cycles: u64,
    pub display: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT],
    pub keyboard: [bool; KEYBOARD_SIZE],
    pub state: Chip8State,
    pub random: ChaCha8Rng,
}

#[derive(Debug)]
struct DecodedInstruction {
    position3: u8,
    position2: u8,
    position1: u8,
    position0: u8,
    address: u16,
    nibble: usize,
    x: usize,
    y: usize,
    byte: u8,
}

impl Chip8MachineState {
    pub fn new() -> Self {
        Self {
            cycles: 0,
            display: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
            keyboard: [false; KEYBOARD_SIZE],
            state: Chip8State::new(),
            random: ChaCha8Rng::from_os_rng(),
        }
    }

    pub fn execute_cycle(&mut self) {
        let (address, instruction) = self.fetch_instruction();

        let decoded = self.decode_instruction(instruction);

        self.state.pc += 2;

        let result = self.execute_instruction(&decoded);

        if !result {
            panic!("Unknown or invalid instruction at {address:#X}: {instruction:X?}")
        }

        self.cycles += 1;
    }

    fn fetch_instruction(&self) -> (usize, [u8; 2]) {
        let address : usize = self.state.pc as usize;

        if address >= self.state.memory.len() {
            panic!("Program counter out of bounds: {address:#X}");
        }

        let instruction: [u8; 2] = if address + 1 < self.state.memory.len() {
            self.state.memory[address..address + 2].try_into().unwrap()
        } else {
            [self.state.memory[address], self.state.memory[0]]
        };

        (address, instruction)
    }

    fn decode_instruction(&self, instruction: [u8; 2]) -> DecodedInstruction {
        let position3 = instruction[0] >> 4;
        let position2 = instruction[0] & 0x0F;
        let position1 = instruction[1] >> 4;
        let position0 = instruction[1] & 0x0F;
        let address = ((instruction[0] as u16 & 0x0F) << 8) | (instruction[1] as u16);
        let nibble = position0 as usize;
        let x = position2 as usize;
        let y = position1 as usize;
        let byte = instruction[1];

        DecodedInstruction {
            position3,
            position2,
            position1,
            position0,
            address,
            nibble,
            x,
            y,
            byte,
        }
    }

    fn execute_instruction(&mut self, decoded: &DecodedInstruction) -> bool {
        let mut result = true;

        match (decoded.position3, decoded.position2, decoded.position1, decoded.position0) {
            (0x0, 0x0, 0xE, 0x0) => self.execute_cls(),
            (0x0, 0x0, 0xE, 0xE) => self.execute_ret(),
            (0x0, _, _, _) => self.execute_sys_addr(decoded.address),
            (0x1, _, _, _) => self.execute_jp_addr(decoded.address),
            (0x2, _, _, _) => self.execute_call_addr(decoded.address),
            (0x3, _, _, _) => self.execute_se_vx_byte(decoded.x, decoded.byte),
            (0x4, _, _, _) => self.execute_sne_vx_byte(decoded.x, decoded.byte),
            (0x5, _, _, 0x0) => self.execute_se_vx_vy(decoded.x, decoded.y),
            (0x6, _, _, _) => self.execute_ld_vx_byte(decoded.x, decoded.byte),
            (0x7, _, _, _) => self.execute_add_vx_byte(decoded.x, decoded.byte),
            (0x8, _, _, 0x0) => self.execute_ld_vx_vy(decoded.x, decoded.y),
            (0x8, _, _, 0x1) => self.execute_or_vx_vy(decoded.x, decoded.y),
            (0x8, _, _, 0x2) => self.execute_and_vx_vy(decoded.x, decoded.y),
            (0x8, _, _, 0x3) => self.execute_xor_vx_vy(decoded.x, decoded.y),
            (0x8, _, _, 0x4) => self.execute_add_vx_vy(decoded.x, decoded.y),
            (0x8, _, _, 0x5) => self.execute_sub_vx_vy(decoded.x, decoded.y),
            (0x8, _, _, 0x6) => self.execute_shr_vx(decoded.x),
            (0x8, _, _, 0x7) => self.execute_subn_vx_vy(decoded.x, decoded.y),
            (0x8, _, _, 0xE) => self.execute_shl_vx(decoded.x),
            (0x9, _, _, 0x0) => self.execute_sne_vx_vy(decoded.x, decoded.y),
            (0xA, _, _, _) => self.execute_ld_i_addr(decoded.address),
            (0xD, _, _, _) => self.execute_draw_vx_vy_nibble(decoded.x, decoded.y, decoded.nibble),
            (0xB, _, _, _) => self.execute_jp_v0_addr(decoded.address),
            (0xC, _, _, _) => self.execute_rnd_vx_byte(decoded.x, decoded.byte),
            (0xE, _, 0x9, 0xE) => self.execute_skp_vx(decoded.x),
            (0xE, _, 0xA, 0x1) => self.execute_sknp_vx(decoded.x),
            (0xF, _, 0x0, 0x7) => self.execute_ld_vx_dt(decoded.x),
            (0xF, _, 0x0, 0xA) => self.execute_ld_vx_k(decoded.x),
            (0xF, _, 0x1, 0x5) => self.execute_ld_dt_vx(decoded.x),
            (0xF, _, 0x1, 0x8) => self.execute_ld_st_vx(decoded.x),
            (0xF, _, 0x1, 0xE) => self.execute_add_i_vx(decoded.x),
            (0xF, _, 0x2, 0x9) => self.execute_ld_f_vx(decoded.x),
            (0xF, _, 0x3, 0x3) => self.execute_ld_b_vx(decoded.x),
            (0xF, _, 0x5, 0x5) => self.execute_ld_ref_i_vx(),
            (0xF, _, 0x6, 0x5) => self.execute_ld_vx_ref_i(),
            (..) => result = false,
        }

        result
    }

    fn execute_cls(&mut self) {
        self.display.iter_mut().for_each(|row| row.fill(false));
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
        self.state.v[x] = self.random.random::<u8>() & byte;
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

    pub fn set_pixel(&mut self, x: usize, y: usize, on: bool) {
        self.display[y][x] = on;
    }
    
    pub fn get_pixel(&self, x: usize, y: usize) -> bool {
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