use ggez;
use ggez::event;
use ggez::event::KeyCode;
use ggez::event::KeyMods;
use ggez::graphics;
use ggez::graphics::Rect;

use rand::Rng;

use std::env;
use std::fs::File;
use std::io::prelude::*;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const PIXEL_SIZE: i32 = 20;
const WINDOW_WIDTH: f32 = SCREEN_WIDTH as f32 * PIXEL_SIZE as f32;
const WINDOW_HEIGHT: f32 = SCREEN_HEIGHT as f32 * PIXEL_SIZE as f32;

struct Chip8State {
    memory: [u8; 4096],
    pc: u16,
    registers: [u8; 16],
    i_register: u16,
    stack: [u16; 12],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    keys: [bool; 16],
    video_buf: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT],
}

impl Chip8State {
    fn new(rom: &str) -> ggez::GameResult<Chip8State> {
        let mut s = Chip8State {
            memory: [0; 4096],
            pc: 512,
            registers: [0; 16],
            i_register: 0,
            stack: [0; 12],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keys: [false; 16],
            video_buf: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
        };

        s.load_sprites();

        s.load_rom(rom);

        Ok(s)
    }

    fn load_sprites(&mut self) {
        let sprites = [
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

        for i in 0..sprites.len() {
            self.memory[i] = sprites[i];
        }
    }

    fn load_rom(&mut self, path: &str) {
        let mut file = File::open(path).unwrap();

        file.read(&mut self.memory[512..]).unwrap();
    }

    fn get_opcode(&self) -> u16 {
        (self.memory[self.pc as usize] as u16) << 8 | (self.memory[(self.pc + 1) as usize] as u16)
    }

    fn process_opcode(&mut self, opcode: u16) {
        let nibble_one = (opcode & 0xF000) >> 12;
        let nibble_two = (opcode & 0x0F00) >> 8;
        let nibble_three = (opcode & 0x00F0) >> 4;
        let nibble_four = opcode & 0x000F;

        let reg_x = nibble_two as usize;
        let reg_y = nibble_three as usize;
        let opcode_nn = (opcode & 0x00FF) as u8;
        let address = opcode & 0x0FFF;

        print!("pc: {:#05X}, {:04X} : ", self.pc, opcode);

        match (nibble_one, nibble_two, nibble_three, nibble_four) {
            (0, 0, 0xE, 0) => {
                println!("Clear the screen");
                for row in 0..SCREEN_HEIGHT {
                    for col in 0..SCREEN_WIDTH {
                        self.video_buf[row][col] = false;
                    }
                }
            }
            (0, 0, 0xE, 0xE) => {
                println!("Return from a subroutine");
                self.sp -= 1;
                self.pc = self.stack[self.sp as usize];
                return;
            }
            (0, _, _, _) => {
                println!(
                    "Execute machine language subroutine at address {:#05X}",
                    address
                );
                panic!("{:04X} not implemented", opcode);
            }
            (1, _, _, _) => {
                println!("Jump to address {:#05X}", address);
                self.pc = address;
                return;
            }
            (2, _, _, _) => {
                println!("Execute subroutine starting at address {:#05X}", address);
                self.stack[self.sp as usize] = self.pc + 2;
                self.sp += 1;
                self.pc = address;
                return;
            }
            (3, _, _, _) => {
                println!(
                    "Skip the following instruction if the value of register V{:X} equals {:#04X}",
                    reg_x, opcode_nn
                );
                if self.registers[reg_x] == opcode_nn {
                    self.pc += 2;
                }
            }
            (4, _, _, _) => {
                println!("Skip the following instruction if the value of register V{:X} is not equal to {:#04X}", reg_x, opcode_nn);
                if self.registers[reg_x] != opcode_nn {
                    self.pc += 2;
                }
            }
            (5, _, _, _) => {
                println!("Skip the following instruction if the value of register V{:X} is equal to the value of register V{:X}", reg_x, reg_y);
                if self.registers[reg_x] == self.registers[reg_y] {
                    self.pc += 2;
                }
            }
            (6, _, _, _) => {
                println!("Store number {:#04X} in register V{:X}", opcode_nn, reg_x);
                self.registers[reg_x] = opcode_nn;
            }
            (7, _, _, _) => {
                println!("Add the value {:#04X} to register V{:X}", opcode_nn, reg_x);
                self.registers[reg_x] += opcode_nn;
            }
            (8, _, _, 0) => {
                println!(
                    "Store the value of register V{:X} in register V{:X}",
                    reg_y, reg_x
                );
                self.registers[reg_x] = self.registers[reg_y];
            }
            (8, _, _, 1) => {
                println!("Set V{0:X} to V{0:X} OR V{1:X}", reg_x, reg_y);
                self.registers[reg_x] |= self.registers[reg_y];
            }
            (8, _, _, 2) => {
                println!("Set V{0:X} to V{0:X} AND V{1:X}", reg_x, reg_y);
                self.registers[reg_x] &= self.registers[reg_y];
            }
            (8, _, _, 3) => {
                println!("Set V{0:X} to V{0:X} XOR V{1:X}", reg_x, reg_y);
                self.registers[reg_x] ^= self.registers[reg_y];
            }
            (8, _, _, 4) => {
                println!(
                    "Add the value of register V{:X} to register V{:X}",
                    reg_y, reg_x
                );
                println!("Set VF to 01 if borrow occurs");
                println!("Set VF to 00 if borrow does not occur");
                if self.registers[reg_x] as u16 + self.registers[reg_y] as u16 > 0xFF {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
                self.registers[reg_x] += self.registers[reg_y];
            }
            (8, _, _, 5) => {
                println!(
                    "Subtract the value of register V{:X} from register V{:X}",
                    reg_y, reg_x
                );
                println!("Set VF to 00 if borrow occurs");
                println!("Set VF to 01 if borrow does not occur");
                if self.registers[reg_x] > self.registers[reg_y] {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
                self.registers[reg_x] -= self.registers[reg_y];
            }
            (8, _, _, 6) => {
                println!(
                    "Store the value of register V{:X} shifted right one bit in register V{:X}",
                    reg_y, reg_x
                );
                println!("Set register VF to the least significant bit prior to the shift");
                self.registers[0xF] = self.registers[reg_x] & 0x1;
                self.registers[reg_x] = self.registers[reg_y] >> 1;
            }
            (8, _, _, 7) => {
                println!(
                    "Set register V{0:X} to the value of V{1:X} minus V{0:X}",
                    reg_x, reg_y
                );
                println!("Set VF to 00 if a borrow occurs");
                println!("Set VF to 01 if a borrow does not occur");
                if self.registers[reg_y] > self.registers[reg_x] {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
                self.registers[reg_x] = self.registers[reg_y] - self.registers[reg_x];
            }
            (8, _, _, 0xE) => {
                println!(
                    "Store the value of register V{:X} shifted left one bit in register V{:X}",
                    reg_y, reg_x
                );
                println!("Set register VF to the most significant bit prior to the shift");
                self.registers[0xF] = (self.registers[reg_x] & 0x8) >> 7;
                self.registers[reg_x] = self.registers[reg_y] << 1;
            }
            (9, _, _, 0) => {
                println!("Skip the following instruction if the value of register V{:X} is not equal to the value of register V{:X}", reg_x, reg_y);
                if self.registers[reg_x] != self.registers[reg_y] {
                    self.pc += 2;
                }
            }
            (0xA, _, _, _) => {
                println!("Store memory address {:#05X} in register I", address);
                self.i_register = address;
            }
            (0xB, _, _, _) => {
                println!("Jump to address {:#05X} + V0", address);
                self.pc = address + self.registers[0] as u16;
                return;
            }
            (0xC, _, _, _) => {
                println!(
                    "Set V{:X} to a random number with a mask of {:#04X}",
                    reg_x, opcode_nn
                );
                let random_num = rand::thread_rng().gen_range(0x00, 0xFF);
                self.registers[reg_x] = random_num & opcode_nn;
            }
            (0xD, _, _, _) => {
                println!("Draw a sprite at position V{:X}, V{:X} with {} bytes of sprite data starting at the address stored in I", reg_x, reg_y, opcode & 0x000F);
                println!("Set VF to 01 if any set pixels are changed to unset, and 00 otherwise");
                let sprite_rows = (opcode & 0x000F) as u8;
                self.registers[0xF] = 0;
                for sprite_row in 0..sprite_rows {
                    for sprite_col in 0..8 {
                        let screen_row: usize = (self.registers[reg_y] + sprite_row) as usize;
                        let screen_col: usize = (self.registers[reg_x] + sprite_col) as usize;
                        let sprite_pixel = (self.memory
                            [(self.i_register + sprite_row as u16) as usize]
                            & (0x80 >> sprite_col))
                            >> 7 - sprite_col;

                        if screen_row < SCREEN_HEIGHT && screen_col < SCREEN_WIDTH {
                            if sprite_pixel == 1 {
                                if self.video_buf[screen_row][screen_col] == true {
                                    self.registers[0xF] = 1;
                                }
                                self.video_buf[screen_row][screen_col] ^= true;
                            }
                        }
                    }
                }
            }
            (0xE, _, 9, 0xE) => {
                println!("Skip the following instruction if the key corresponding to the hex value currently stored in register V{:X} is pressed", reg_x);
                if self.keys[self.registers[reg_x] as usize] == true {
                    self.pc += 2;
                }
            }
            (0xE, _, 0xA, 1) => {
                println!("Skip the following instruction if the key corresponding to the hex value currently stored in register V{:X} is not pressed", reg_x);
                if self.keys[self.registers[reg_x] as usize] == false {
                    self.pc += 2;
                }
            }
            (0xF, _, 0, 7) => {
                println!(
                    "Store the current value of the delay timer in register V{:X}",
                    reg_x
                );
                self.registers[reg_x] = self.delay_timer;
            }
            (0xF, _, 0, 0xA) => {
                println!(
                    "Wait for a key press and store the result in register V{:X}",
                    reg_x
                );
                let mut key_pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] == true {
                        self.registers[reg_x] = i as u8;
                        key_pressed = true;
                        break;
                    }
                }
                if !key_pressed {
                    return;
                }
            }
            (0xF, _, 1, 5) => {
                println!("Set the delay timer to the value of register V{:X}", reg_x);
                self.delay_timer = self.registers[reg_x];
            }
            (0xF, _, 1, 8) => {
                println!("Set the sound timer to the value of register V{:X}", reg_x);
                self.sound_timer = self.registers[reg_x];
            }
            (0xF, _, 1, 0xE) => {
                println!(
                    "Add the value stored in register V{:X} to register I",
                    reg_x
                );
                self.i_register += self.registers[reg_x] as u16;
            }
            (0xF, _, 2, 9) => {
                println!("Set I to the memory address of the sprite data corresponding to the hexadecimal digit stored in register V{:X}", reg_x);
                self.i_register = (self.registers[reg_x] * 5) as u16;
            }
            (0xF, _, 3, 3) => {
                println!("Store the binary-coded decimal equivalent of the value stored in register V{:X} at addresses I, I+1, and I+2", reg_x);
                self.memory[self.i_register as usize] = (self.registers[reg_x] / 100) % 10; // Hundreds
                self.memory[self.i_register as usize + 1] = (self.registers[reg_x] / 10) % 10; // Tens
                self.memory[self.i_register as usize + 2] = self.registers[reg_x] % 10; // Ones
            }
            (0xF, _, 5, 5) => {
                println!("Store the values of registers V0 to V{:X} inclusive in memory starting at address I", reg_x);
                println!("I is set to I + {:X} + 1 after operation", reg_x);
                for i in 0..reg_x + 1 {
                    self.memory[self.i_register as usize + i] = self.registers[i];
                }
                self.i_register += reg_x as u16 + 1;
            }
            (0xF, _, 6, 5) => {
                println!("Fill registers V0 to V{:X} inclusive with the values stored in memory starting at address I", reg_x);
                println!("I is set to I + {:X} + 1 after operation", reg_x);
                for i in 0..reg_x + 1 {
                    self.registers[i] = self.memory[self.i_register as usize + i];
                }
                self.i_register += reg_x as u16 + 1;
            }
            _ => {
                panic!("Unknown opcode: {:04X}", opcode);
            }
        }

        self.pc += 2;
    }
}

impl event::EventHandler for Chip8State {
    fn update(&mut self, _ctx: &mut ggez::Context) -> ggez::GameResult {
        let opcode = self.get_opcode();
        self.process_opcode(opcode);

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            // TODO: BEEP!
            println!("BEEP!");
            self.sound_timer -= 1;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());

        for row in 0..SCREEN_HEIGHT {
            for col in 0..SCREEN_WIDTH {
                if self.video_buf[row][col] == true {
                    let color = [1.0, 1.0, 1.0, 1.0].into();
                    let rect = Rect::new_i32(
                        col as i32 * PIXEL_SIZE,
                        row as i32 * PIXEL_SIZE,
                        PIXEL_SIZE,
                        PIXEL_SIZE,
                    );
                    let rectangle = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        rect,
                        color,
                    )?;
                    graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
                }
            }
        }

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut ggez::Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::Key1 => {
                self.keys[0x1] = true;
            }
            KeyCode::Key2 => {
                self.keys[0x2] = true;
            }
            KeyCode::Key3 => {
                self.keys[0x3] = true;
            }
            KeyCode::Key4 => {
                self.keys[0xC] = true;
            }
            KeyCode::Q => {
                self.keys[0x4] = true;
            }
            KeyCode::W => {
                self.keys[0x5] = true;
            }
            KeyCode::E => {
                self.keys[0x6] = true;
            }
            KeyCode::R => {
                self.keys[0xD] = true;
            }
            KeyCode::A => {
                self.keys[0x7] = true;
            }
            KeyCode::S => {
                self.keys[0x8] = true;
            }
            KeyCode::D => {
                self.keys[0x9] = true;
            }
            KeyCode::F => {
                self.keys[0xE] = true;
            }
            KeyCode::Z => {
                self.keys[0xA] = true;
            }
            KeyCode::X => {
                self.keys[0x0] = true;
            }
            KeyCode::C => {
                self.keys[0xB] = true;
            }
            KeyCode::V => {
                self.keys[0xF] = true;
            }
            _ => (),
        }
    }

    fn key_up_event(&mut self, _ctx: &mut ggez::Context, keycode: KeyCode, _keymods: KeyMods) {
        match keycode {
            KeyCode::Key1 => {
                self.keys[0x1] = false;
            }
            KeyCode::Key2 => {
                self.keys[0x2] = false;
            }
            KeyCode::Key3 => {
                self.keys[0x3] = false;
            }
            KeyCode::Key4 => {
                self.keys[0xC] = false;
            }
            KeyCode::Q => {
                self.keys[0x4] = false;
            }
            KeyCode::W => {
                self.keys[0x5] = false;
            }
            KeyCode::E => {
                self.keys[0x6] = false;
            }
            KeyCode::R => {
                self.keys[0xD] = false;
            }
            KeyCode::A => {
                self.keys[0x7] = false;
            }
            KeyCode::S => {
                self.keys[0x8] = false;
            }
            KeyCode::D => {
                self.keys[0x9] = false;
            }
            KeyCode::F => {
                self.keys[0xE] = false;
            }
            KeyCode::Z => {
                self.keys[0xA] = false;
            }
            KeyCode::X => {
                self.keys[0x0] = false;
            }
            KeyCode::C => {
                self.keys[0xB] = false;
            }
            KeyCode::V => {
                self.keys[0xF] = false;
            }
            _ => (),
        }
    }
}

fn main() -> ggez::GameResult {
    let args: Vec<String> = env::args().collect();
    let (ctx, event_loop) = &mut ggez::ContextBuilder::new("CHIP-8", "Ryan Hope")
        .window_setup(ggez::conf::WindowSetup::default().title("CHIP-8"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build()?;
    let state = &mut Chip8State::new(&args[1])?;
    event::run(ctx, event_loop, state)
}
