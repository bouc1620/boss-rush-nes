use super::bus::ADDR_PRG_ROM;
use super::cartridge::{Cartridge, CartridgeState};

pub trait Mapper {
    fn cpu_read(&self, addr: usize, cart: &Cartridge) -> u8;
    fn cpu_write(&mut self, addr: usize, data: u8, cart: &mut CartridgeState);

    fn ppu_read(&self, addr: usize, cart: &Cartridge) -> u8;
    fn ppu_write(&mut self, addr: usize, data: u8, cart: &mut CartridgeState);
}

pub enum MapperKind {
    Nrom(NromMapper),
}

impl MapperKind {
    pub fn from_id(mapper_id: u8) -> Option<Self> {
        match mapper_id {
            0 => Some(MapperKind::Nrom(NromMapper {})),
            _ => None,
        }
    }
}

macro_rules! delegate_mapper {
    ($self:ident, $method:ident $(, $arg:expr )*) => {
        match $self {
            MapperKind::Nrom(inner) => inner.$method($($arg),*),
        }
    };
}

impl Mapper for MapperKind {
    fn cpu_read(&self, addr: usize, cart: &Cartridge) -> u8 {
        delegate_mapper!(self, cpu_read, addr, cart)
    }

    fn cpu_write(&mut self, addr: usize, data: u8, cart: &mut CartridgeState) {
        delegate_mapper!(self, cpu_write, addr, data, cart)
    }

    fn ppu_read(&self, addr: usize, cart: &Cartridge) -> u8 {
        delegate_mapper!(self, ppu_read, addr, cart)
    }

    fn ppu_write(&mut self, addr: usize, data: u8, cart: &mut CartridgeState) {
        delegate_mapper!(self, ppu_write, addr, data, cart)
    }
}

pub struct NromMapper {}

impl Mapper for NromMapper {
    fn cpu_read(&self, addr: usize, cart: &Cartridge) -> u8 {
        let offset = addr - ADDR_PRG_ROM;
        let mapped_addr = if cart.nb_prg_banks == 1 {
            offset & 0x3FFF
        } else {
            offset
        };

        cart.prg_rom.get(mapped_addr).copied().unwrap_or(0)
    }

    fn cpu_write(&mut self, addr: usize, data: u8, cart: &mut CartridgeState) {
        todo!()
    }

    fn ppu_read(&self, addr: usize, cart: &Cartridge) -> u8 {
        todo!()
    }

    fn ppu_write(&mut self, addr: usize, data: u8, cart: &mut CartridgeState) {
        todo!()
    }
}
