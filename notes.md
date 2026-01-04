## Ownership / application DAG

```
Nes (owns everything)
 ├─ Cpu      (state machine)
 ├─ Ppu      (state machine)
 └─ Bus      (wiring + memory map)
     ├─ RAM
     ├─ Cartridge
     ├─ PPU registers
     └─ APU
```

## Interrupts

Peut-être possible d'utiliser les enums et le pattern matching pour les interrupts.
Ou encore pour les mappers.

## Bus

Adresses sur le bus principal:

- RAM: 0x0000 - 0x1FFF - Mirrored 3 fois sur le premier 2Kb.  
  Donc les plages d'adresses: 0x0800-0x0FFF, 0x1000-0x17FF et 0x1800-0x1FFF sont mappées sur 0x0000
- 0x07FF les premiers 2Kb, Suffit de mod 0x07FF
- 0x2000 - 0x401F ???
- Program ROM: 0x4020
- 0xFFFF (Cartridge) \*
- Mapper - APU - Controls - Other

Adresses sur le bus du PPU:

- Pattern memory: 0x0000 - 0x1FFF (Cartridge)
- Name table: 0x2000 - 0x2FFF
- 0x3000 - 0x3EFF ??? <<<<<<
- Palettes: 0x3F00 - 0x3FFF
- Program ROM: 0x4020 - 0xFFFF (Cartridge) \*

---

```
mod x;
		Private
		Visible to this module + its children
pub mod x;
		Visible to the parent
		Reachable from anywhere through a public path
pub(crate) mod x;
		Visible anywhere in this crate
		Hidden from other crates
```
