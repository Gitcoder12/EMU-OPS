use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Cpu {
    pub pc: u16,
    pub sp: u8,
    pub i: u16,
    pub v: [u8; 16],
    pub stack: [u16; 16],
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub opcode: u16,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            pc: 0x200,
            sp: 0,
            i: 0,
            v: [0; 16],
            stack: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            opcode: 0,
        }
    }

    pub fn tick(&mut self) {
        if self.delay_timer > 0 { self.delay_timer -= 1; }
        if self.sound_timer > 0 { self.sound_timer -= 1; }
    }

    pub fn fetch(&mut self, memory: &[u8; 4096]) {
        let high = memory[self.pc as usize] as u16;
        let low = memory[self.pc as usize + 1] as u16;
        self.opcode = (high << 8) | low;
        self.pc += 2;
    }

    pub fn execute(&mut self, memory: &mut [u8; 4096], display: &mut [[bool; 64]; 32]) -> bool {
        let op = self.opcode;
        let x = ((op & 0x0F00) >> 8) as usize;
        let y = ((op & 0x00F0) >> 4) as usize;
        let n = (op & 0x000F) as u8;
        let kk = (op & 0x00FF) as u8;
        let nnn = op & 0x0FFF;

        match op & 0xF000 {
            0x0000 => match op {
                0x00E0 => {
                    for row in display.iter_mut() {
                        for pixel in row.iter_mut() {
                            *pixel = false;
                        }
                    }
                }
                0x00EE => {
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize];
                }
                _ => {}
            },
            0x1000 => { self.pc = nnn; }
            0x2000 => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            }
            0x3000 => if self.v[x] == kk { self.pc += 2; },
            0x4000 => if self.v[x] != kk { self.pc += 2; },
            0x5000 => if self.v[x] == self.v[y] { self.pc += 2; },
            0x6000 => { self.v[x] = kk; }
            0x7000 => { self.v[x] = self.v[x].wrapping_add(kk); }
            0x8000 => match op & 0x000F {
                0x0000 => self.v[x] = self.v[y],
                0x0001 => self.v[x] |= self.v[y],
                0x0002 => self.v[x] &= self.v[y],
                0x0003 => self.v[x] ^= self.v[y],
                0x0004 => {
                    let (val, overflow) = self.v[x].overflowing_add(self.v[y]);
                    self.v[x] = val;
                    self.v[0xF] = overflow as u8;
                }
                0x0005 => {
                    let (val, borrow) = self.v[x].overflowing_sub(self.v[y]);
                    self.v[x] = val;
                    self.v[0xF] = (!borrow) as u8;
                }
                0x0006 => {
                    self.v[0xF] = self.v[x] & 0x1;
                    self.v[x] >>= 1;
                }
                0x0007 => {
                    let (val, borrow) = self.v[y].overflowing_sub(self.v[x]);
                    self.v[x] = val;
                    self.v[0xF] = (!borrow) as u8;
                }
                0x000E => {
                    self.v[0xF] = (self.v[x] >> 7) & 0x1;
                    self.v[x] <<= 1;
                }
                _ => {}
            },
            0x9000 => if self.v[x] != self.v[y] { self.pc += 2; },
            0xA000 => { self.i = nnn; }
            0xB000 => { self.pc = (nnn as u16) + (self.v[0] as u16); }
            0xC000 => {
                let random: u8 = rand::random();
                self.v[x] = random & kk;
            }
            0xD000 => {
                let x_pos = self.v[x] as usize % 64;
                let y_pos = self.v[y] as usize % 32;
                let height = n;
                self.v[0xF] = 0;
                for row in 0..height {
                    let sprite_byte = memory[(self.i + row as u16) as usize];
                    for col in 0..8 {
                        if (sprite_byte >> (7 - col)) & 1 == 1 {
                            let px = (x_pos + col) % 64;
                            let py = (y_pos + row as usize) % 32;
                            if display[py][px] {
                                self.v[0xF] = 1;
                            }
                            display[py][px] ^= true;
                        }
                    }
                }
            }
            0xE000 => match kk {
                0x9E => { /* skip if key pressed – needs keyboard input */ }
                0xA1 => { /* skip if key not pressed */ }
                _ => {}
            },
            0xF000 => match kk {
                0x07 => self.v[x] = self.delay_timer,
                0x0A => { /* wait for key – advanced */ }
                0x15 => self.delay_timer = self.v[x],
                0x18 => self.sound_timer = self.v[x],
                0x1E => self.i += self.v[x] as u16,
                0x29 => self.i = (self.v[x] as u16) * 5,
                0x33 => {
                    let val = self.v[x];
                    memory[self.i as usize] = val / 100;
                    memory[self.i as usize + 1] = (val / 10) % 10;
                    memory[self.i as usize + 2] = val % 10;
                }
                0x55 => {
                    for idx in 0..=x {
                        memory[(self.i + idx as u16) as usize] = self.v[idx];
                    }
                }
                0x65 => {
                    for idx in 0..=x {
                        self.v[idx] = memory[(self.i + idx as u16) as usize];
                    }
                }
                _ => {}
            },
            _ => {}
        }
        true
    }
}
