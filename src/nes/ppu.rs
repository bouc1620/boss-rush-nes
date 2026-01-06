pub struct Ppu {
    pub pattern: [u8; 8 * 1024],
    pub name_table: [u8; 2 * 1024],
    pub palette: [u8; 32],
    pub registers: [u8; 8],
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            pattern: [0; 8 * 1024],
            name_table: [0; 2 * 1024],
            palette: [0; 32],
            registers: [0; 8],
        }
    }
}
