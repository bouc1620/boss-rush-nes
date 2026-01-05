use super::cartridge::Cartridge;
use super::ppu::Ppu;
use std::cell::RefCell;
use std::rc::Rc;

pub const ADDR_PRG_ROM: usize = 0x8000;
pub const ADDR_RESET_VECTOR: usize = 0xFFFC;

pub struct Bus {
    ram: [u8; 64 * 1024],
    ppu: Rc<RefCell<Ppu>>,
    cartridge: Rc<RefCell<Cartridge>>,
}

impl Bus {
    pub fn new(ppu: Rc<RefCell<Ppu>>, cartridge: Rc<RefCell<Cartridge>>) -> Self {
        Self {
            ram: [0; 64 * 1024],
            ppu,
            cartridge,
        }
    }
}

impl Bus {
    pub fn cpu_read(&self, addr: u16, _readonly: bool) -> u8 {
        if addr < 0x2000 {
            // Internal RAM: 0x0000 - 0x1FFF (mirrored 3 times)
            let addr = addr & 0x07FF;
            self.ram[addr as usize]
        } else if addr < 0x4000 {
            // PPU registers: $2000 - $3FFF (mirrored every 8 bytes)
            let addr = 0x2000 + (addr & 0x0007);
            self.ppu.borrow_mut().registers[addr as usize]
        } else if addr < 0x4017 {
            // APU / IO: $4000 - $4017
            self.ram[addr as usize]
        } else if addr >= ADDR_PRG_ROM as u16 {
            // Cartridge PRG-ROM: 0x8000 - 0xFFFF
            let offset = (addr as usize) - ADDR_PRG_ROM;
            self.cartridge.borrow().read(offset)
        } else {
            0
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        if addr < 0x2000 {
            // Internal RAM: 0x0000 - 0x1FFF (mirrored 3 times)
            let addr = addr & 0x07FF;
            self.ram[addr as usize] = data;
        }
    }

    pub fn ppu_read(&self, addr: u16, _readonly: bool) -> u8 {
        if addr < 0x2000 {
            // Internal RAM: 0x0000 - 0x1FFF (mirrored 3 times)
            let addr = addr & 0x07FF;
            self.ram[addr as usize]
        } else {
            0
        }
    }

    pub fn ppu_write(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }
}

#[cfg(feature = "debug")]
impl Bus {
    pub fn print_ram(&self, start: u16, end: u16) {
        for addr in start..=end {
            if (addr - start).is_multiple_of(16) {
                print!("\n{:04X}: ", addr);
            }

            print!("{:02X} ", self.cpu_read(addr, true));
        }

        println!();
    }
}
