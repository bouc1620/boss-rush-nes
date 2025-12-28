pub struct CPU {
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    p: u8,
    memory: [u8; 0x1000],
}

impl CPU {
    pub fn new(bus: &super::bus::Bus) -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0,
            pc: 0,
            p: 0,
            memory: [0; 0x1000],
        }
    }
}

const FLAG_CARRY: u8 = 0b000_0001;
const FLAG_ZERO: u8 = 0b000_0010;
const FLAG_INTERRUPT_DISABLE: u8 = 0b000_0100;
const FLAG_DECIMAL: u8 = 0b000_1000;
const FLAG_BREAK: u8 = 0b001_0000;
const FLAG_UNUSED: u8 = 0b010_0000;
const FLAG_OVERFLOW: u8 = 0b0100_0000;
const FLAG_NEGATIVE: u8 = 0b1000_0000;

impl CPU {
    fn BRK(&mut self) -> u8 {
        0
    }
    fn ORA(&mut self) -> u8 {
        0
    }
    fn XXX(&mut self) -> u8 {
        0
    }
    fn NOP(&mut self) -> u8 {
        0
    }
    fn ASL(&mut self) -> u8 {
        0
    }
    fn PHP(&mut self) -> u8 {
        0
    }
    fn BPL(&mut self) -> u8 {
        0
    }
    fn CLC(&mut self) -> u8 {
        0
    }
    fn JSR(&mut self) -> u8 {
        0
    }
    fn AND(&mut self) -> u8 {
        0
    }
    fn BIT(&mut self) -> u8 {
        0
    }
    fn ROL(&mut self) -> u8 {
        0
    }
    fn PLP(&mut self) -> u8 {
        0
    }
    fn BMI(&mut self) -> u8 {
        0
    }
    fn SEC(&mut self) -> u8 {
        0
    }
    fn RTI(&mut self) -> u8 {
        0
    }
    fn EOR(&mut self) -> u8 {
        0
    }
    fn LSR(&mut self) -> u8 {
        0
    }
    fn PHA(&mut self) -> u8 {
        0
    }
    fn JMP(&mut self) -> u8 {
        0
    }
    fn BVC(&mut self) -> u8 {
        0
    }
    fn CLI(&mut self) -> u8 {
        0
    }
    fn RTS(&mut self) -> u8 {
        0
    }
    fn ADC(&mut self) -> u8 {
        0
    }
    fn ROR(&mut self) -> u8 {
        0
    }
    fn PLA(&mut self) -> u8 {
        0
    }
    fn BVS(&mut self) -> u8 {
        0
    }
    fn SEI(&mut self) -> u8 {
        0
    }
    fn STA(&mut self) -> u8 {
        0
    }
    fn STY(&mut self) -> u8 {
        0
    }
    fn STX(&mut self) -> u8 {
        0
    }
    fn DEY(&mut self) -> u8 {
        0
    }
    fn TXA(&mut self) -> u8 {
        0
    }
    fn BCC(&mut self) -> u8 {
        0
    }
    fn TYA(&mut self) -> u8 {
        0
    }
    fn TXS(&mut self) -> u8 {
        0
    }
    fn LDY(&mut self) -> u8 {
        0
    }
    fn LDA(&mut self) -> u8 {
        0
    }
    fn LDX(&mut self) -> u8 {
        0
    }
    fn TAY(&mut self) -> u8 {
        0
    }
    fn TAX(&mut self) -> u8 {
        0
    }
    fn BCS(&mut self) -> u8 {
        0
    }
    fn CLV(&mut self) -> u8 {
        0
    }
    fn TSX(&mut self) -> u8 {
        0
    }
    fn CPY(&mut self) -> u8 {
        0
    }
    fn CMP(&mut self) -> u8 {
        0
    }
    fn DEC(&mut self) -> u8 {
        0
    }
    fn INY(&mut self) -> u8 {
        0
    }
    fn DEX(&mut self) -> u8 {
        0
    }
    fn BNE(&mut self) -> u8 {
        0
    }
    fn CLD(&mut self) -> u8 {
        0
    }
    fn CPX(&mut self) -> u8 {
        0
    }
    fn SBC(&mut self) -> u8 {
        0
    }
    fn INC(&mut self) -> u8 {
        0
    }
    fn INX(&mut self) -> u8 {
        0
    }
    fn BEQ(&mut self) -> u8 {
        0
    }
    fn SED(&mut self) -> u8 {
        0
    }
}
