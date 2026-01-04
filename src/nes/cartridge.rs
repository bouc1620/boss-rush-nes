use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

#[derive(Debug, Copy, Clone)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    FourScreen,
}

pub struct Cartridge {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub mirroring: Mirroring,
    pub prg_ram_size: u8,
}

fn get_mirroring(flag6: u8) -> Mirroring {
    match (flag6 & 0x08 != 0, flag6 & 0x01 != 0) {
        (true, _) => Mirroring::FourScreen,
        (false, true) => Mirroring::Vertical,
        (false, false) => Mirroring::Horizontal,
    }
}

impl Cartridge {
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        if &buffer[0..4] != b"NES\x1A" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid NES header",
            ));
        }

        let prg_size = buffer[4] as usize * 16 * 1024;
        let chr_size = buffer[5] as usize * 8 * 1024;

        let flag6 = buffer[6];
        let flag7 = buffer[7];

        let mapper = (flag7 & 0xF0) | (flag6 >> 4);

        let mirroring = get_mirroring(flag6);

        let prg_ram_size = buffer[8];

        let prg_rom = buffer[16..16 + prg_size].to_vec();
        let chr_rom = buffer[16 + prg_size..16 + prg_size + chr_size].to_vec();

        Ok(Self {
            prg_rom,
            chr_rom,
            mapper,
            mirroring,
            prg_ram_size,
        })
    }
}
