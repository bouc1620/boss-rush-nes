use crate::bus::Bus;
use crate::instructions::{INSTRUCTIONS, Instruction};

#[derive(Default)]
pub struct CPU {
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    p: u8,

    fetched: u8,   // Data fetched from memory
    addr_abs: u16, // Absolute address calculated by addressing mode
    addr_rel: u16, // Relative address offset for branch instructions
    opcode: u8,    // Current instruction opcode
    cycles: u8,    // Remaining clock cycles
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
    fn current_instruction(&self) -> Instruction {
        let row = (self.opcode / 16) as usize;
        let col = (self.opcode % 16) as usize;
        INSTRUCTIONS[row][col]
    }

    fn get_flag(&self, flag: u8) -> u8 {
        if self.p & flag != 0 { 1 } else { 0 }
    }

    fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.p |= flag;
        } else {
            self.p &= !flag;
        }
    }

    pub fn reset(&mut self, bus: &mut Bus) {
        self.addr_abs = 0xFFFC;

        let lo = bus.read(self.addr_abs, false) as u16;
        let hi = bus.read(self.addr_abs + 1, false) as u16;
        self.pc = (hi << 8) | lo;

        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFD;
        self.p = FLAG_UNUSED;

        self.addr_abs = 0;
        self.addr_rel = 0;
        self.fetched = 0;

        self.cycles = 8;
    }

    pub fn step(&mut self, bus: &mut Bus) {
        if self.cycles != 0 {
            self.cycles -= 1;
            return;
        }

        self.opcode = bus.read(self.pc, false);
        self.pc += 1;

        let instruction = self.current_instruction();

        self.cycles = instruction.cycles;

        // Add +1 cycle only if a page is crossed and the opcode is a read (eligible for the penalty)
        self.cycles += (instruction.addr)(self, bus) & (instruction.operate)(self, bus);
    }

    // Interrupt request
    pub fn irq(&mut self, _bus: &mut Bus) -> u8 {
        0 // TODO: rendu ici
    }

    pub fn brk(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn ora(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn nop(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn asl(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn php(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn bpl(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn clc(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn jsr(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn and(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn bit(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn rol(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn plp(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn bmi(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn sec(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn rti(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn eor(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn lsr(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn pha(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn jmp(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn bvc(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn cli(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn rts(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn adc(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn ror(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn pla(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn bvs(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn sei(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn sta(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn sty(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn stx(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn dey(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn txa(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn bcc(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn tya(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn txs(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn ldy(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn lda(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn ldx(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn tay(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn tax(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn bcs(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn clv(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn tsx(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn cpy(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn cmp(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn dec(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn iny(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn dex(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn bne(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn cld(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn cpx(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn sbc(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn inc(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn inx(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn beq(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn sed(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn zp0(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn imm(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn izx(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn imp(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn abs(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn rel(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn izy(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn zpy(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn zpx(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn aby(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn abx(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn ind(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn xxx(&mut self, _bus: &mut Bus) -> u8 {
        0
    }
}
