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
    stack: Vec<u16>,
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
            stack: vec![],
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

    pub fn run_cycle(&mut self) {
        self.opcode = u16::from(self.memory[usize::from(self.pc)]) << 8
            | u16::from(self.memory[usize::from(self.pc + 1)]);
        println!("{:x?}", self.opcode);
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
                self.pc = self.stack[0];
                self.sp -= 1;
            }
            (0x0, n1, n2, n3) => {}
            (0x1, n1, n2, n3) => {
                self.pc = (n1 << 8) + (n2 << 4) + n3;
            }
            (0x2, n1, n2, n3) => {
                self.sp += 1;
                self.stack.push(self.pc);
                self.pc = (n1 << 8) + (n2 << 4) + n3;
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
                self.registers[x as usize] += ((k1 << 4) + k2) as u8;
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
                    (rx + ry - 0x100) as u8
                } else {
                    self.registers[15] = 0;
                    (rx + ry) as u8
                };
            }
            (0x8, x, y, 0x5) => {
                let rx = self.registers[x as usize];
                let ry = self.registers[y as usize];
                self.registers[x as usize] = if rx > ry {
                    self.registers[15] = 1;
                    rx - ry
                } else {
                    self.registers[15] = 0;
                    0
                }
            }
            (0x8, x, y, 0x6) => {
                let rx = self.registers[x as usize];
                if rx % 2 == 1 {
                    self.registers[15] = 1;
                } else {
                    self.registers[15] = 0;
                };
                self.registers[x as usize] = rx / 2;
            }
            (0x8, x, y, 0x7) => {
                let rx = self.registers[x as usize];
                let ry = self.registers[y as usize];
                self.registers[x as usize] = if ry > rx {
                    self.registers[15] = 1;
                    ry - rx
                } else {
                    self.registers[15] = 0;
                    0
                }
            }
            (0x8, x, y, 0xE) => {
                let rx = self.registers[x as usize];
                if rx >> 7 == 1 {
                    self.registers[15] = 1;
                } else {
                    self.registers[15] = 0;
                };
                self.registers[x as usize] = if u16::from(rx) * 2 > 0xFF {
                    0xFF
                } else {
                    rx * 2
                };
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
                self.pc = (n1 << 8) + (n2 << 4) + n3 + u16::from(self.registers[0]);
            }
            (0xC, x, k1, k2) => {
                let mut rng = rand::thread_rng();
                self.registers[x as usize] = rng.gen::<u8>() & ((k1 << 4) + k2) as u8;
            }
            (0xD, x, y, n) => {}
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
            (0xF, x, 0x0, 0xA) => {}
            (0xF, x, 0x1, 0x5) => {}
            (0xF, x, 0x1, 0x8) => {}
            (0xF, x, 0x1, 0xE) => {}
            (0xF, x, 0x2, 0x9) => {}
            (0xF, x, 0x3, 0x3) => {}
            (0xF, x, 0x5, 0x5) => {}
            (0xF, x, 0x6, 0x5) => {}
            opcode => {
                println!("This opcode does not exist : {:?}", opcode);
            }
        };

        self.pc += 2;
    }
}
