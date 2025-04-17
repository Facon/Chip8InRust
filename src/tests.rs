#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use crate::chip8_machine::{Chip8MachineState, SCREEN_HEIGHT, SCREEN_WIDTH};
    use crate::chip8_state::Chip8State;
    use crate::disassembly::DisassemblyOutput;

    #[test]
    fn should_create() {
        let mut cpu = Chip8State::new();

        cpu.memory[5] = 3;

        println!("{:?}", cpu);
    }

    #[test]
    fn should_print_instruction() {
        let cpu = Chip8State::new();

        println!("{}", cpu.disassemble(0));
    }

    #[test]
    fn should_print_all_instructions() {
        let cpu = Chip8State::with_all_instructions();

        for address in (0x200..0x248).step_by(2) {
            println!("{}", cpu.disassemble(address));
        }
    }

    #[test]
    fn should_execute_cls() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x00, 0xE0]);
        chip8.display[0][0] = true;
        chip8.display[0][1] = true;
        chip8.display[1][1] = true;
        chip8.display[20][20] = true;
        chip8.display[SCREEN_HEIGHT - 1][SCREEN_WIDTH - 1] = true;

        chip8.execute_cycle();

        assert!(!chip8.display[0][0]);
        assert!(!chip8.display[0][1]);
        assert!(!chip8.display[1][1]);
        assert!(!chip8.display[20][20]);
        assert!(!chip8.display[SCREEN_HEIGHT - 1][SCREEN_WIDTH - 1]);
    }

    #[test]
    fn should_execute_ret() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x00, 0xEE]);
        chip8.state.stack[0] = 0;
        chip8.state.sp = 1;
        chip8.state.pc = 0x200;

        chip8.execute_cycle();

        assert_eq!(chip8.state.sp, 0);
        assert_eq!(chip8.state.pc, 0x000);
    }

    #[test]
    #[ignore = "NOP instruction"]
    fn should_execute_sys_addr() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x02, 0x00]);

        chip8.execute_cycle();

        assert_eq!(chip8.state.sp, 1);
        assert_eq!(chip8.state.stack[0], 0x202);
        assert_eq!(chip8.state.pc, 0x200);
    }

    #[test]
    fn should_execute_jp_addr() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x12, 0x00]);

        chip8.execute_cycle();

        assert_eq!(chip8.state.pc, 0x200);
    }

    #[test]
    fn should_execute_call_addr() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x22, 0x00]);

        chip8.execute_cycle();

        assert_eq!(chip8.state.sp, 1);
        assert_eq!(chip8.state.stack[0], 0x202);
        assert_eq!(chip8.state.pc, 0x200);
    }

    #[test]
    fn should_execute_call_addr_do_sum_then_ret() {
        let mut chip8 = Chip8MachineState::new();

        /*
        0x200: call 0x300 # 0x2300
        0x300: ld v0, 0x1 # 0x6001
        0x302: ld v1, 0x1 # 0x6101
        0x304: add v0, v1 # 0x8014
        0x306: ret # 0x00EE
        */
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x23, 0x00]);
        chip8.state.memory[0x300..0x308]
            .clone_from_slice(&[0x60, 0x01, 0x61, 0x01, 0x80, 0x14, 0x00, 0xEE]);

        chip8.execute_cycle();

        assert_eq!(chip8.state.sp, 1);
        assert_eq!(chip8.state.stack[0], 0x202);
        assert_eq!(chip8.state.pc, 0x300);

        chip8.execute_cycle();
        chip8.execute_cycle();
        chip8.execute_cycle();
        chip8.execute_cycle();

        assert_eq!(chip8.state.sp, 0);
        assert_eq!(chip8.state.stack[0], 0x202);
        assert_eq!(chip8.state.pc, 0x202);
    }

    #[test]
    fn should_execute_se_vx_byte_should_not_skip() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x30, 0xFF]);

        chip8.execute_cycle();

        assert_eq!(chip8.state.pc, 0x202);
    }

    #[test]
    fn should_execute_se_vx_byte_should_skip() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x30, 0xFF]);
        chip8.state.v[0] = 0xFF;

        chip8.execute_cycle();

        assert_eq!(chip8.state.pc, 0x204);
    }

    #[test]
    fn should_execute_sne_vx_byte_should_not_skip() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x40, 0xFF]);
        chip8.state.v[0] = 0xFF;

        chip8.execute_cycle();

        assert_eq!(chip8.state.pc, 0x202);
    }

    #[test]
    fn should_execute_sne_vx_byte_should_skip() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x40, 0xFF]);

        chip8.execute_cycle();

        assert_eq!(chip8.state.pc, 0x204);
    }

    #[test]
    fn should_execute_se_vx_vy_should_not_skip() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x50, 0x20]);
        chip8.state.v[0] = 0xFF;
        chip8.state.v[2] = 0x00;

        chip8.execute_cycle();

        assert_eq!(chip8.state.pc, 0x202);
    }

    #[test]
    fn should_execute_se_vx_vy_should_skip() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x50, 0x20]);
        chip8.state.v[0] = 0xFF;
        chip8.state.v[2] = 0xFF;

        chip8.execute_cycle();

        assert_eq!(chip8.state.pc, 0x204);
    }

    #[test]
    fn should_execute_ld_vx_byte() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x63, 0x11]);

        chip8.execute_cycle();

        assert_eq!(chip8.state.v[3], 0x11);
    }

    #[test]
    fn should_execute_add_vx_byte() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x7E, 0x11]);
        chip8.state.v[0xE] = 0x11;

        chip8.execute_cycle();

        assert_eq!(chip8.state.v[0xE], 0x22);
    }

    #[test]
    fn should_execute_ld_vx_vy() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x84, 0x50]);
        chip8.state.v[0x5] = 0xAA;

        chip8.execute_cycle();

        assert_eq!(chip8.state.v[0x4], 0xAA);
    }

    #[test]
    fn should_execute_or_vx_vy() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x84, 0x51]);
        chip8.state.v[0x4] = 0x0B;
        chip8.state.v[0x5] = 0xB0;

        chip8.execute_cycle();

        assert_eq!(chip8.state.v[0x4], 0xBB);
    }

    #[test]
    fn should_execute_and_vx_vy() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x85, 0x62]);
        chip8.state.v[0x5] = 0x1B;
        chip8.state.v[0x6] = 0xB0;

        chip8.execute_cycle();

        assert_eq!(chip8.state.v[0x5], 0x10);
    }

    #[test]
    fn should_execute_xor_vx_vy() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x86, 0x73]);
        chip8.state.v[0x6] = 0xB0;
        chip8.state.v[0x7] = 0xBB;

        chip8.execute_cycle();

        assert_eq!(chip8.state.v[0x6], 0x0B);
    }

    #[test]
    fn should_execute_add_vx_vy_should_not_get_carry() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x8C, 0xD4]);
        chip8.state.v[0xC] = 0x01;
        chip8.state.v[0xD] = 0x01;

        chip8.execute_cycle();

        assert_eq!(chip8.state.v[0xC], 0x02);
        assert_eq!(chip8.state.v[0xF], 0x00);
    }

    #[test]
    fn should_execute_add_vx_vy_should_get_carry() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x8A, 0xB4]);
        chip8.state.v[0xA] = 0xFF;
        chip8.state.v[0xB] = 0xFF;

        chip8.execute_cycle();

        assert_eq!(chip8.state.v[0xA], 0xFE);
        assert_eq!(chip8.state.v[0xF], 0x01);
    }

    #[test]
    fn should_execute_sub_vx_vy_should_get_borrow() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x88, 0x95]);
        chip8.state.v[0x8] = 0x0F;
        chip8.state.v[0x9] = 0x0E;

        chip8.execute_cycle();

        assert_eq!(chip8.state.v[0x8], 0x01);
        assert_eq!(chip8.state.v[0xF], 0x01);
    }

    #[test]
    fn should_execute_sub_vx_vy_should_not_get_borrow() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x86, 0x75]);
        chip8.state.v[0x6] = 0xFE;
        chip8.state.v[0x7] = 0xFF;

        chip8.execute_cycle();

        assert_eq!(chip8.state.v[0x6], 0xFF);
        assert_eq!(chip8.state.v[0xF], 0x00);
    }

    #[test]
    fn should_execute_shr_vx_should_not_set_vf() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x85, 0x06]);
        chip8.state.v[0x5] = 0xFE;

        chip8.execute_cycle();

        assert_eq!(chip8.state.v[0x5], 0x7F);
        assert_eq!(chip8.state.v[0xF], 0x00);
    }

    #[test]
    fn should_execute_shr_vx_should_set_vf() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x85, 0x06]);
        chip8.state.v[0x5] = 0xFF;

        chip8.execute_cycle();

        assert_eq!(chip8.state.v[0x5], 0x7F);
        assert_eq!(chip8.state.v[0xF], 0x01);
    }

    #[test]
    fn should_execute_subn_vx_vy_should_get_borrow() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x81, 0x27]);
        chip8.state.v[0x1] = 0x0E;
        chip8.state.v[0x2] = 0x0F;

        chip8.execute_cycle();

        assert_eq!(chip8.state.v[0x1], 0x01);
        assert_eq!(chip8.state.v[0xF], 0x01);
    }

    #[test]
    fn should_execute_subn_vx_vy_should_not_get_borrow() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x81, 0x27]);
        chip8.state.v[0x1] = 0x0F;
        chip8.state.v[0x2] = 0x0E;

        chip8.execute_cycle();

        assert_eq!(chip8.state.v[0x1], 0xFF);
        assert_eq!(chip8.state.v[0xF], 0x00);
    }

    #[test]
    fn should_execute_shl_vx() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x81, 0x2E]);
        chip8.state.v[0x1] = 0x84;

        chip8.execute_cycle();

        assert_eq!(chip8.state.v[0x1], 0x08);
        assert_eq!(chip8.state.v[0xF], 0x01);
    }

    #[test]
    fn should_execute_sne_vx_vy_should_not_skip() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x9E, 0xF0]);
        chip8.state.v[0xE] = 0x00;
        chip8.state.v[0xF] = 0x00;

        chip8.execute_cycle();

        assert_eq!(chip8.state.pc, 0x202);
    }

    #[test]
    fn should_execute_sne_vx_vy_should_skip() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0x9E, 0xF0]);
        chip8.state.v[0xE] = 0x01;
        chip8.state.v[0xF] = 0x00;

        chip8.execute_cycle();

        assert_eq!(chip8.state.pc, 0x204);
    }

    #[test]
    fn should_execute_ld_i_addr() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0xA2, 0x34]);

        chip8.execute_cycle();

        assert_eq!(chip8.state.i, 0x0234);
    }

    #[test]
    fn should_execute_jp_v0_addr() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0xB3, 0x45]);
        chip8.state.v[0x0] = 0x10;

        chip8.execute_cycle();

        assert_eq!(chip8.state.pc, 0x0355);
    }

    #[test]
    fn should_execute_rnd_vx_byte() {
        let mut chip8 = Chip8MachineState::new();
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0xC3, 0xFF]);
        chip8.random = ChaCha8Rng::seed_from_u64(2); // Create fixed number, only for this test!

        chip8.execute_cycle();

        assert_eq!(chip8.state.v[0x3], 0xC5);
    }

    #[test]
    fn should_set_and_get_pixel() {
        let mut chip8 = Chip8MachineState::new();
        chip8.set_pixel(50, 25, true);

        assert!(chip8.get_pixel(50, 25));
    }

    #[test]
    fn should_execute_draw_vx_vy_nibble() {
        let mut chip8 = Chip8MachineState::new();
        let sprite_location: u16 = 0x300;
        let sprite_size: u8 = 0x3;
        chip8.state.memory[0x200..0x202].clone_from_slice(&[0xD5, 0x60 | sprite_size]);
        chip8.state.i = sprite_location;
        chip8.state.memory[sprite_location as usize..0x300 | sprite_size as usize]
            .clone_from_slice(&[0xFF, 0xFF, 0xFF]);
        chip8.state.v[0x5] = 0;
        chip8.state.v[0x6] = 0;

        chip8.execute_cycle();

        let mut display: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT] =
            [[false; SCREEN_WIDTH]; SCREEN_HEIGHT];

        display[0][0] = true;
        display[0][1] = true;
        display[0][2] = true;
        display[0][3] = true;
        display[0][4] = true;
        display[0][5] = true;
        display[0][6] = true;
        display[0][7] = true;
        display[1][0] = true;
        display[1][1] = true;
        display[1][2] = true;
        display[1][3] = true;
        display[1][4] = true;
        display[1][5] = true;
        display[1][6] = true;
        display[1][7] = true;
        display[2][0] = true;
        display[2][1] = true;
        display[2][2] = true;
        display[2][3] = true;
        display[2][4] = true;
        display[2][5] = true;
        display[2][6] = true;
        display[2][7] = true;

        assert_eq!(chip8.display, display);
    }

    /*
    #[test]
    fn should_execute_
     */
}
