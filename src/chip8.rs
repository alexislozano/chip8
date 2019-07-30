use rand::Rng;

const FONT: [u8; 80] = [
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

pub struct Chip8 {
    opcode: u16,
    memory: [u8; 4096],
    registers: [u8; 16],
    i: u16,
    pc: u16,
    display: [[bool; 64]; 32],
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    sp: usize,
    keys: [bool; 16],
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            opcode: 0,
            memory: [0; 4096],
            registers: [0; 16],
            i: 0,
            pc: 0x200,
            display: [[false; 64]; 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            keys: [false; 16],
        }
    }

    pub fn load_font(&mut self) {
        self.memory[0..80].clone_from_slice(&FONT[0..80])
    }

    pub fn load_game(&mut self, bytes: Vec<u8>) {
        for (i, b) in bytes.iter().enumerate() {
            self.memory[i + 0x200] = *b;
        }
    }

    pub fn set_key(&mut self, key: u8, val: bool) {
        self.keys[key as usize] = val;
    }

    pub fn display(&self) -> [[bool; 64]; 32] {
        self.display
    }

    pub fn sound_timer(&self) -> u8 {
        self.sound_timer
    }

    pub fn run_cycle(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
        self.opcode = u16::from(self.memory[usize::from(self.pc)]) << 8
            | u16::from(self.memory[usize::from(self.pc + 1)]);
        match (
            self.opcode >> 12,
            (self.opcode >> 8) % 16,
            (self.opcode >> 4) % 16,
            self.opcode % 16,
        ) {
            (0x0, 0x0, 0xE, 0x0) => {
                self.display = [[false; 64]; 32];
            }
            (0x0, 0x0, 0xE, 0xE) => {
                self.pc = self.stack[self.sp];
                self.sp -= 1;
            }
            (0x0, 0x0, 0x0, 0x0) => {}
            (0x1, n1, n2, n3) => {
                self.pc = (n1 << 8) + (n2 << 4) + n3 - 2;
            }
            (0x2, n1, n2, n3) => {
                self.sp += 1;
                self.stack[self.sp] = self.pc;
                self.pc = (n1 << 8) + (n2 << 4) + n3 - 2;
            }
            (0x3, x, k1, k2) => {
                if self.registers[x as usize] == ((k1 << 4) + k2) as u8 {
                    self.pc += 2;
                }
            }
            (0x4, x, k1, k2) => {
                if self.registers[x as usize] != ((k1 << 4) + k2) as u8 {
                    self.pc += 2;
                }
            }
            (0x5, x, y, 0x0) => {
                if self.registers[x as usize] == self.registers[y as usize] {
                    self.pc += 2;
                }
            }
            (0x6, x, k1, k2) => {
                self.registers[x as usize] = ((k1 << 4) + k2) as u8;
            }
            (0x7, x, k1, k2) => {
                let rx = u16::from(self.registers[x as usize]);
                let k = (k1 << 4) + k2;
                self.registers[x as usize] = if rx + k > 0xFF {
                    (rx + k) % 0x100
                } else {
                    rx + k
                } as u8;
            }
            (0x8, x, y, 0x0) => {
                self.registers[x as usize] = self.registers[y as usize];
            }
            (0x8, x, y, 0x1) => {
                self.registers[x as usize] |= self.registers[y as usize];
            }
            (0x8, x, y, 0x2) => {
                self.registers[x as usize] &= self.registers[y as usize];
            }
            (0x8, x, y, 0x3) => {
                self.registers[x as usize] ^= self.registers[y as usize];
            }
            (0x8, x, y, 0x4) => {
                let rx = u16::from(self.registers[x as usize]);
                let ry = u16::from(self.registers[y as usize]);
                self.registers[x as usize] = if rx + ry > 0x100 {
                    self.registers[15] = 1;
                    (rx + ry) % 0x100
                } else {
                    self.registers[15] = 0;
                    rx + ry
                } as u8;
            }
            (0x8, x, y, 0x5) => {
                let rx = u16::from(self.registers[x as usize]);
                let ry = u16::from(self.registers[y as usize]);
                self.registers[x as usize] = if rx > ry {
                    self.registers[15] = 1;
                    rx - ry
                } else {
                    self.registers[15] = 0;
                    0x100 - (ry - rx)
                } as u8;
            }
            (0x8, x, _y, 0x6) => {
                let rx = self.registers[x as usize];
                if rx % 2 == 1 {
                    self.registers[15] = 1;
                } else {
                    self.registers[15] = 0;
                };
                self.registers[x as usize] = rx / 2;
            }
            (0x8, x, y, 0x7) => {
                let rx = u16::from(self.registers[x as usize]);
                let ry = u16::from(self.registers[y as usize]);
                self.registers[x as usize] = if ry > rx {
                    self.registers[15] = 1;
                    ry - rx
                } else {
                    self.registers[15] = 0;
                    0x100 - (rx - ry)
                } as u8
            }
            (0x8, x, _y, 0xE) => {
                let rx = u16::from(self.registers[x as usize]);
                if rx >> 7 == 1 {
                    self.registers[15] = 1;
                } else {
                    self.registers[15] = 0;
                };
                self.registers[x as usize] = if rx * 2 > 0xFF {
                    (rx * 2) % 0x100
                } else {
                    rx * 2
                } as u8;
            }
            (0x9, x, y, 0x0) => {
                if self.registers[x as usize] != self.registers[y as usize] {
                    self.pc += 2;
                }
            }
            (0xA, n1, n2, n3) => {
                self.i = (n1 << 8) + (n2 << 4) + n3;
            }
            (0xB, n1, n2, n3) => {
                self.pc = (n1 << 8) + (n2 << 4) + n3 + u16::from(self.registers[0]) - 2;
            }
            (0xC, x, k1, k2) => {
                let mut rng = rand::thread_rng();
                let r = rng.gen::<u8>();
                self.registers[x as usize] = r & ((k1 << 4) + k2) as u8;
            }
            (0xD, x, y, n) => {
                let rx = u16::from(self.registers[x as usize]);
                let ry = u16::from(self.registers[y as usize]);
                let sprite = &self.memory[self.i as usize..(self.i + n) as usize];
                let mut collision = false;
                for h in 0..n {
                    for w in 0..8 {
                        let new_w = (rx + w) % 64;
                        let new_h = (ry + h) % 32;
                        let old_pixel = self.display[new_h as usize][new_w as usize];
                        let new_pixel = (sprite[h as usize] << w >> 7) == 1;
                        self.display[new_h as usize][new_w as usize] = old_pixel ^ new_pixel;
                        if old_pixel && !self.display[new_h as usize][new_w as usize] {
                            collision = true;
                        }
                    }
                }
                self.registers[15] = collision as u8;
            }
            (0xE, x, 0x9, 0xE) => {
                if self.keys[self.registers[x as usize] as usize] {
                    self.pc += 2;
                }
            }
            (0xE, x, 0xA, 0x1) => {
                if !self.keys[self.registers[x as usize] as usize] {
                    self.pc += 2;
                }
            }
            (0xF, x, 0x0, 0x7) => {
                self.registers[x as usize] = self.delay_timer;
            }
            (0xF, x, 0x0, 0xA) => {
                match self.keys.iter().position(|k| *k) {
                    Some(i) => self.registers[x as usize] = i as u8,
                    None => self.pc -= 2,
                };
            }
            (0xF, x, 0x1, 0x5) => {
                self.delay_timer = self.registers[x as usize];
            }
            (0xF, x, 0x1, 0x8) => {
                self.sound_timer = self.registers[x as usize];
            }
            (0xF, x, 0x1, 0xE) => {
                self.i += u16::from(self.registers[x as usize]);
            }
            (0xF, x, 0x2, 0x9) => {
                self.i = u16::from(self.registers[x as usize]) * 5;
            }
            (0xF, x, 0x3, 0x3) => {
                let rx = self.registers[x as usize];
                let (k1, k2, k3) = (rx / 100, rx / 10 % 10, rx % 10);
                self.memory[self.i as usize] = k1;
                self.memory[(self.i + 1) as usize] = k2;
                self.memory[(self.i + 2) as usize] = k3;
            }
            (0xF, x, 0x5, 0x5) => {
                self.memory[self.i as usize..(self.i + x + 1) as usize]
                    .clone_from_slice(&self.registers[0..(x + 1) as usize]);
            }
            (0xF, x, 0x6, 0x5) => {
                self.registers[0..(x + 1) as usize]
                    .clone_from_slice(&self.memory[self.i as usize..(self.i + x + 1) as usize]);
            }
            opcode => {
                println!("This opcode does not exist : {:?}", opcode);
            }
        };

        self.pc += 2;
    }
}
