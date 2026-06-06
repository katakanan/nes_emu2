use std::cell::Cell;
use std::ops::{Coroutine, CoroutineState};
use std::pin::Pin;

use crate::pad;

use super::cpu::{debug_op, debug_reg, Cpu, CpuStep};
use super::pad::Pad;
use super::ppu::*;
use nes_rom_reader::cassette::Cassette;

pub struct Nes {
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub rom: Cassette,
    pub ram: Cell<[u8; 0x0800]>,
    pub pad: Pad,
}

#[derive(Debug)]
pub enum NesStep {
    Ppu(PpuStep),
    Cpu(CpuStep),
}

impl Nes {
    pub fn run<'a>(&'a self) -> impl Coroutine<Yield = NesStep, Return = !> + 'a {
        let mut nes_cpu = Cpu::run(self);
        let mut nes_ppu = Ppu::run(self);

        let mut cpu_cycle = 7;
        let mut cpu_cycle_before_exec = cpu_cycle;
        let mut ppu_cycle = cpu_cycle * 3;
        let mut ppu_cycle_before_exec = ppu_cycle;
        let mut ppu_line = 0;
        let mut ppu_line_before_exec = ppu_line;
        let mut regs_before_exec: Cpu = self.cpu.clone();

        #[coroutine] move || loop {
            loop {
                match Pin::new(&mut nes_cpu).resume(()) {
                    CoroutineState::Yielded(cpu_step @ CpuStep::Cycle) => {
                        if cfg!(feature = "nestest") {
                            cpu_cycle = cpu_cycle + 1;
                        }
                        yield NesStep::Cpu(cpu_step);
                        break;
                    }
                    CoroutineState::Yielded(operation) => {
                        //Debug
                        let debug_line = if cfg!(feature = "nestest") {
                            let line = format!(
                                "{:<48}{} PPU:{:>3},{:>3} CYC:{}",
                                debug_op(&operation),
                                debug_reg(&regs_before_exec),
                                ppu_line_before_exec,
                                ppu_cycle_before_exec,
                                cpu_cycle_before_exec
                            );
                            cpu_cycle_before_exec = cpu_cycle;
                            ppu_cycle_before_exec = ppu_cycle;
                            ppu_line_before_exec = ppu_line;
                            regs_before_exec = self.cpu.clone();
                            line
                        } else {
                            "".to_string()
                        };
                        // yield NesStep::Cpu(operation);
                        yield NesStep::Cpu(CpuStep::DebugLine(debug_line))
                    }
                }
            }

            for _ in 0..3 {
                loop {
                    match Pin::new(&mut nes_ppu).resume(()) {
                        CoroutineState::Yielded(ppu_step @ PpuStep::Cycle(_x, _y)) => {
                            if cfg!(feature = "nestest") {
                                ppu_cycle = ppu_cycle + 1;

                                if ppu_cycle > 340 {
                                    ppu_cycle = 0;
                                    ppu_line = ppu_line + 1;
                                }
                            }
                            yield NesStep::Ppu(ppu_step);
                            break;
                        }
                        CoroutineState::Yielded(ppu_step) => {
                            yield NesStep::Ppu(ppu_step);
                        }
                    }
                }
            }
        }
    }

    pub fn start(path: &str) -> Self {
        let cpu = Cpu::power_up();
        let rom = Cassette::load(path).unwrap();

        let mut tmp = rom.chrrom.clone();
        tmp.resize(0x2000, 0);
        let chrrom: [u8; 0x2000] = tmp.try_into().unwrap();

        let mirror = if rom.mapper1 & 0x01 == 0x01 {
            MIRROR::VERTICAL
        } else {
            MIRROR::HORIZONTAL
        };

        let ppu = Ppu::power_up(chrrom, mirror);
        let ram = Cell::new([0; 0x0800]);
        let pad = Pad::default();
        let nes = Nes {
            cpu,
            ppu,
            rom,
            ram,
            pad,
        };
        nes.cpu.pc.set(nes.read16(0xFFFC));
        nes
    }

    pub fn read8(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x07FF => {
                //WRAM
                self.ram.get()[addr as usize]
            }
            0x0800..=0x1FFF => {
                //mirror to WRAM
                let offset = (addr - 0x0800) % 0x800;
                self.ram.get()[offset as usize]
            }
            0x2000..=0x2007 => {
                //PPU Regs
                self.ppu.read_reg(addr & 0x0F)
            }
            0x2008..=0x3FFF => {
                let offset = (addr - 0x2008) % 8;
                self.ppu.read_reg(offset)
            }
            0x8000..=0xFFFF => {
                let offset = (addr - 0x8000) % self.rom.prgrom_size as u16;
                self.rom.read8(offset)
            }
            0x4015 => {
                0xFF //To pass nestest.log
            }
            0x4016 => {
                let s = self.pad.shift_index.get();
                if s > 7 {
                    return 1;
                }

                self.pad.shift_index.set(s + 1);

                let btn = if !self.pad.strobe_enable.get() {
                    let scanbtn = pad::PadButton::from_bits_truncate(0x01 << s);
                    self.pad.status.get().contains(scanbtn)
                } else {
                    true
                };

                btn as u8
            }
            0x4017 => 0,
            0x4000..=0x401F => {
                //APU I/O, PAD
                0xFF //To pass nestest.log
            }
            _ => {
                unimplemented!("Not Cpu Ram access to @0x{:04X}", addr);
            }
        }
    }

    fn copy_oam_data(&self, page: u8) {
        //should be re-write
        let target_addr_start = self.ppu.oamaddr.get() as u16;
        let mut oam = self.ppu.oam.get();

        for i in 0x00..=0xFF {
            let source_addr = ((page as u16) << 8) | i;
            let byte = self.read8(source_addr);

            let target_addr = (target_addr_start + i) as usize % oam.len();
            oam[target_addr] = byte;
        }

        self.ppu.oam.set(oam);
    }

    pub fn write8(&self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x07FF => {
                //WRAM
                self.ram()[addr as usize].set(data);
            }
            0x0800..=0x1FFF => {
                //mirror to WRAM
                let offset = (addr - 0x0800) % (0x7FF);
                self.ram()[offset as usize].set(data);
            }
            0x2000..=0x2007 => {
                //PPU Regs
                // self.ppu./
                self.ppu.write_reg(addr & 0x0F, data);
            }
            0x2008..=0x3FFF => {
                let offset = (addr - 0x2008) % 8;
                self.ppu.write_reg(offset, data);
            }
            0x4014 => {
                self.copy_oam_data(data);
                // OAMDMA stalls CPU for 513 cycles (514 if started on odd CPU cycle).
                // Use 513 as a baseline; this is critical for game timing.
                self.cpu.dma_stall.set(513);
            }
            0x4016 => {
                let strobe = (data & 0x01) != 0;

                if strobe {
                    self.pad.strobe_enable.set(true);
                } else {
                    self.pad.shift_index.set(0);
                    self.pad.strobe_enable.set(false);
                }
            }
            0x4000..=0x401F => {
                //APU I/O, 2nd PAD
            }
            0x4020..=0x5FFF => {
                unimplemented!("Extend ROM @0x{:04X}", addr);
            }
            0x6000..=0x7FFF => {
                unimplemented!("Extend ROM @0x{:04X}", addr);
            }
            0x8000..=0xFFFF => {
                unreachable!("ROM is Read Only");
            }
        }
    }

    pub fn read16(&self, addr: u16) -> u16 {
        let lo = self.read8(addr);
        let hi = self.read8(addr + 1);

        (lo as u16) | ((hi as u16) << 8)
    }

    pub fn ram(&self) -> &[Cell<u8>] {
        let ram: &Cell<[u8]> = &self.ram;
        ram.as_slice_of_cells()
    }
}
