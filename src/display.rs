use crate::chip8_machine::{Chip8MachineState, KEYBOARD_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH};
use minifb::{Key, Window, WindowOptions};

const SCALE: usize = 10;
const WINDOW_WIDTH: usize = SCREEN_WIDTH * SCALE;
const WINDOW_HEIGHT: usize = SCREEN_HEIGHT * SCALE;
const KEY_MAP: [(Key, usize); KEYBOARD_SIZE] = [
    (Key::X, 0x0),    // 0
    (Key::Key1, 0x1), // 1
    (Key::Key2, 0x2), // 2
    (Key::Key3, 0x3), // 3
    (Key::Q, 0x4),    // 4
    (Key::W, 0x5),    // 5
    (Key::E, 0x6),    // 6
    (Key::A, 0x7),    // 7
    (Key::S, 0x8),    // 8
    (Key::D, 0x9),    // 9
    (Key::Z, 0xA),    // A
    (Key::C, 0xB),    // B
    (Key::Key4, 0xC), // C
    (Key::R, 0xD),    // D
    (Key::F, 0xE),    // E
    (Key::V, 0xF),    // F
];

pub struct Display {
    window: Window,
    buffer: Vec<u32>,
}

impl Display {
    pub fn new(title: &str) -> Self {
        let mut window = Window::new(
            title,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            WindowOptions {
                resize: true,
                scale: minifb::Scale::X1,
                ..WindowOptions::default()
            },
        )
        .expect("Unable to create window");

        // Limit to max ~60 fps update rate
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        Self {
            window,
            buffer: vec![0; WINDOW_WIDTH * WINDOW_HEIGHT],
        }
    }

    pub fn update(&mut self, chip8: &mut Chip8MachineState) -> bool {
        if !self.window.is_open() {
            return false;
        }

        // Update keyboard state
        for (key, value) in KEY_MAP.iter() {
            chip8.set_key(*value, self.window.is_key_down(*key));
        }

        // Clear buffer
        self.buffer.fill(0);

        // Draw scaled pixels
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                if chip8.get_pixel(x, y) {
                    // Draw a SCALE x SCALE pixel
                    for dy in 0..SCALE {
                        for dx in 0..SCALE {
                            let buffer_index = (y * SCALE + dy) * WINDOW_WIDTH + (x * SCALE + dx);
                            self.buffer[buffer_index] = 0xFFFFFF; // White color
                        }
                    }
                }
            }
        }

        // Update window buffer
        self.window
            .update_with_buffer(&self.buffer, WINDOW_WIDTH, WINDOW_HEIGHT)
            .unwrap();

        true
    }
}
