pub struct Bus {
    ram: [u8; 65 * 1024],
}

impl Default for Bus {
    fn default() -> Self {
        Self {
            ram: [0; 65 * 1024],
        }
    }
}

impl Bus {
    pub fn read(&self, addr: u16, _readonly: bool) -> u8 {
        self.ram[addr as usize]
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }
}

#[cfg(feature = "debug")]
impl Bus {
    pub fn load_program(&mut self, program: &str) {
        let program_bytes: Vec<u8> = program
            .split_whitespace()
            .map(|byte| u8::from_str_radix(byte, 16).unwrap())
            .collect();

        self.ram[0x8000..0x8000 + program_bytes.len()].copy_from_slice(&program_bytes);
    }

    pub fn set_reset_vector(&mut self) {
        self.ram[0xFFFC] = 0x00;
        self.ram[0xFFFC + 1] = 0x80;
    }

    pub fn print_ram(&self, start: u16, end: u16) {
        for addr in start..=end {
            if (addr - start).is_multiple_of(16) {
                print!("\n{:04X}: ", addr);
            }

            print!("{:02X} ", self.read(addr, true));
        }

        println!();
    }
}
