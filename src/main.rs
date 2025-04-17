mod chip8_machine;
mod chip8_state;
mod disassembly;
mod display;
pub mod tests;

use chip8_machine::Chip8MachineState;
use display::Display;
use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <rom_path>", args[0]);
        eprintln!("Available ROMs in res/ folder:");
        eprintln!("  - 15puzzle.rom");
        eprintln!("  - blitz.rom");
        eprintln!("  - breakout.rom");
        eprintln!("  - brix.rom");
        eprintln!("  - connect4.rom");
        eprintln!("  - guess.rom");
        eprintln!("  - invaders.rom");
        eprintln!("  - maze.rom");
        eprintln!("  - merlin.rom");
        eprintln!("  - missile.rom");
        eprintln!("  - pong.rom");
        eprintln!("  - tetris.rom");
        process::exit(1);
    }

    let mut chip8 = Chip8MachineState::new();
    chip8.state.load_rom(&args[1], 0x200).unwrap_or_else(|err| {
        eprintln!("Failed to load ROM: {}", err);
        process::exit(2);
    });
    let mut display = Display::new("CHIP-8 Emulator");

    // Main loop
    while display.update(&mut chip8) {
        chip8.execute_cycle();
        // Add any additional logic here (input handling, etc.)
    }
}
