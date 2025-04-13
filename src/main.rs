mod chip8_state;
mod chip8_machine;
mod disassembly;
pub mod tests;

use chip8_state::Chip8State;
use chip8_machine::Chip8MachineState;

fn main() {
    println!("Hello, world!");

    let x = 20;
    let y = 10;

    println!("{}", x + y);
}
