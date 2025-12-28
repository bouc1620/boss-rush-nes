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
