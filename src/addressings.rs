#![allow(unused_doc_comments)]
#![allow(non_snake_case)]
use super::cpu::{Cpu, CpuStep, Status};
use super::nes::Nes;
use super::operations::*;
use std::ops::Coroutine;

#[derive(Debug, Clone, Copy)]
pub enum Addressing {
    Implied,
    Imm,
    Abs,
    AbsX,
    AbsY,
    Zpg,
    ZpgX,
    ZpgY,
    IndY,
    XInd,
    Branch,
}

#[derive(Debug, Clone, Copy)]
pub enum AddressingResult {
    RWValue(u8),
    Address(u16),
    AbsZpgInfo(u16, u8),       //addr, value
    IndInfo(u8, u16, u16, u8), //offset, base, addr, value
    None,
    RegA,
}

#[derive(Debug, Clone)]
pub struct OpArg {
    pub addressing: Addressing,
    pub args: Vec<u8>,
    pub value: AddressingResult,
}

impl Default for OpArg {
    fn default() -> Self {
        OpArg {
            addressing: Addressing::Imm,
            args: vec![],
            value: AddressingResult::None,
        }
    }
}

pub fn Xind_load<'a>(
    nes: &'a Nes,
    op: impl RegOperations + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let base_addr = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let x = nes.cpu.x.get();
        let target_addr = (base_addr as u16 + x as u16) as u8;
        yield CpuStep::Cycle;

        let lo = nes.read8(target_addr as u16);
        yield CpuStep::Cycle;

        let hi = nes.read8((target_addr as u16 + 1) & 0x00FF);
        yield CpuStep::Cycle;

        let addr = (lo as u16) | ((hi as u16) << 8);
        let new_value = nes.read8(addr);
        op.load(nes, new_value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::XInd,
            args: vec![base_addr],
            value: AddressingResult::IndInfo(base_addr, target_addr as u16, addr, new_value),
        }
    }
}

pub fn Zpg_load<'a>(
    nes: &'a Nes,
    op: impl RegOperations + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        //ram[zpg_addr] -> a
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle; //cycle 1 for instruction

        let addr = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle; //cycle 2 for fetch lo_addr

        let new_value = nes.read8(addr as u16);
        op.load(nes, new_value);
        yield CpuStep::Cycle; //cycle 3 for write reg_a
        OpArg {
            addressing: Addressing::Zpg,
            args: vec![addr],
            value: AddressingResult::RWValue(new_value),
        }
    }
}

pub fn Abs_load<'a>(
    nes: &'a Nes,
    op: impl RegOperations + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let lo = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let hi = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let addr = (lo as u16) | ((hi as u16) << 8);
        let new_value = nes.read8(addr);
        op.load(nes, new_value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::Abs,
            args: vec![lo, hi],
            value: AddressingResult::RWValue(new_value),
        }
    }
}

pub fn IndY_load<'a>(
    nes: &'a Nes,
    op: impl RegOperations + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let base_addr = Cpu::fetch_byte_inc_pc(&nes);
        let base_addr_lo = base_addr as u16;
        let base_addr_hi = (base_addr_lo + 1) & 0x00FF;
        yield CpuStep::Cycle;

        let lo = nes.read8(base_addr_lo) as u16;
        yield CpuStep::Cycle;

        let hi = nes.read8(base_addr_hi) as u16;
        yield CpuStep::Cycle;

        let y = nes.cpu.y.get() as u16;
        let tmp_addr = lo | hi << 8;
        let addr = ((lo + y) & 0x00FF) | (hi << 8);
        let effective_addr = (tmp_addr as u32 + y as u32) as u16;
        let addr = if addr == effective_addr {
            addr
        } else {
            yield CpuStep::Cycle;
            effective_addr
        };

        let new_value = nes.read8(addr);
        op.load(nes, new_value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::IndY,
            args: vec![base_addr],
            value: AddressingResult::IndInfo(base_addr, tmp_addr, effective_addr, new_value),
        }
    }
}

pub fn ZpgX_load<'a>(
    nes: &'a Nes,
    op: impl RegOperations + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let lo = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let addr = (lo as u16 + nes.cpu.x.get() as u16) & 0x00FF;
        let new_value = nes.read8(addr);
        yield CpuStep::Cycle;

        op.load(nes, new_value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::ZpgX,
            args: vec![lo],
            value: AddressingResult::AbsZpgInfo(addr, new_value),
        }
    }
}

pub fn ZpgY_load<'a>(
    nes: &'a Nes,
    op: impl RegOperations + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let lo = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let addr = (lo as u16 + nes.cpu.y.get() as u16) & 0x00FF;
        let new_value = nes.read8(addr);
        yield CpuStep::Cycle;

        op.load(nes, new_value);
        yield CpuStep::Cycle;
        OpArg {
            addressing: Addressing::ZpgY,
            args: vec![lo],
            value: AddressingResult::AbsZpgInfo(addr, new_value),
        }
    }
}

pub fn AbsY_load<'a>(
    nes: &'a Nes,
    op: impl RegOperations + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let lo = Cpu::fetch_byte_inc_pc(&nes) as u16;
        yield CpuStep::Cycle;

        let hi = Cpu::fetch_byte_inc_pc(&nes) as u16;
        yield CpuStep::Cycle;

        let y = nes.cpu.y.get() as u16;
        let tmp_addr = lo | hi << 8;
        let addr = (lo + y) | (hi << 8);
        let effective_addr = (tmp_addr as u32 + y as u32) as u16;

        let addr = if addr == effective_addr {
            addr
        } else {
            yield CpuStep::Cycle;
            effective_addr
        };

        let new_value = nes.read8(addr as u16);
        op.load(nes, new_value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::AbsY,
            args: vec![lo as u8, hi as u8],
            value: AddressingResult::AbsZpgInfo(addr, new_value),
        }
    }
}

pub fn AbsX_load<'a>(
    nes: &'a Nes,
    op: impl RegOperations + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let lo = Cpu::fetch_byte_inc_pc(&nes) as u16;
        yield CpuStep::Cycle;

        let hi = Cpu::fetch_byte_inc_pc(&nes) as u16;
        yield CpuStep::Cycle;

        let x = nes.cpu.x.get() as u16;
        let tmp_addr = lo | hi << 8;
        let addr = (lo + x) | (hi << 8);
        let effective_addr = (tmp_addr as u32 + x as u32) as u16;

        let addr = if addr == effective_addr {
            addr
        } else {
            yield CpuStep::Cycle;
            effective_addr
        };

        let new_value = nes.read8(addr as u16);
        op.load(nes, new_value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::AbsX,
            args: vec![lo as u8, hi as u8],
            value: AddressingResult::AbsZpgInfo(addr, new_value),
        }
    }
}

pub fn Imm_load<'a>(
    nes: &'a Nes,
    op: impl RegOperations + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let new_value = Cpu::fetch_byte_inc_pc(&nes);
        op.load(nes, new_value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::Imm,
            args: vec![new_value],
            value: AddressingResult::RWValue(new_value),
        }
    }
}

pub fn Xind_store<'a>(
    nes: &'a Nes,
    op: impl RegOperations + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let base_addr = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let x = nes.cpu.x.get();
        let target_addr = (base_addr as u16 + x as u16) as u8;
        yield CpuStep::Cycle;

        let lo = nes.read8(target_addr as u16);
        yield CpuStep::Cycle;

        let hi = nes.read8((target_addr as u16 + 1) & 0x00FF);
        yield CpuStep::Cycle;

        let addr = (lo as u16) | ((hi as u16) << 8);
        let mut old = 0;

        if cfg!(feature = "nestest") {
            old = nes.read8(addr as u16);
        }

        let value = op.read(nes);
        nes.write8(addr, value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::XInd,
            args: vec![base_addr],
            value: AddressingResult::IndInfo(base_addr, target_addr as u16, addr, old),
        }
    }
}

pub fn Zpg_store<'a>(
    nes: &'a Nes,
    op: impl RegOperations + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let addr = Cpu::fetch_byte_inc_pc(&nes);
        let mut old = 0;

        if cfg!(feature = "nestest") {
            old = nes.read8(addr as u16);
        }

        let value = op.read(nes);
        yield CpuStep::Cycle;

        nes.write8(addr as u16, value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::Zpg,
            args: vec![addr],
            value: AddressingResult::RWValue(old),
        }
    }
}

pub fn Abs_store<'a>(
    nes: &'a Nes,
    op: impl RegOperations + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let lo = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let hi = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let addr = (lo as u16) | ((hi as u16) << 8);
        let mut old = 0;

        if cfg!(feature = "nestest") {
            old = nes.read8(addr as u16);
        }

        let value = op.read(nes);
        nes.write8(addr, value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::Abs,
            args: vec![lo, hi],
            value: AddressingResult::RWValue(old),
        }
    }
}

pub fn IndY_store<'a>(
    nes: &'a Nes,
    op: impl RegOperations + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let base_addr = Cpu::fetch_byte_inc_pc(&nes);
        let base_addr_lo = base_addr as u16;
        let base_addr_hi = (base_addr_lo + 1) & 0x00FF;
        yield CpuStep::Cycle;

        let lo = nes.read8(base_addr_lo) as u16;
        yield CpuStep::Cycle;

        let hi = nes.read8(base_addr_hi) as u16;
        yield CpuStep::Cycle;

        let y = nes.cpu.y.get() as u16;
        let tmp_addr = lo | hi << 8;
        let addr = ((lo + y) & 0x00FF) | (hi << 8);
        let effective_addr = (tmp_addr as u32 + y as u32) as u16;
        yield CpuStep::Cycle;

        let value = op.read(nes);

        let addr = if addr == effective_addr {
            addr
        } else {
            yield CpuStep::Cycle;
            effective_addr
        };

        let mut old = 0;

        if cfg!(feature = "nestest") {
            old = nes.read8(addr as u16);
        }

        nes.write8(addr, value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::IndY,
            args: vec![base_addr],
            value: AddressingResult::IndInfo(base_addr, tmp_addr, effective_addr, old),
        }
    }
}

pub fn ZpgX_store<'a>(
    nes: &'a Nes,
    op: impl RegOperations + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let lo = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let addr = (lo as u16 + nes.cpu.x.get() as u16) & 0x00FF;
        let value = op.read(nes);
        yield CpuStep::Cycle;
        let mut old = 0;

        if cfg!(feature = "nestest") {
            old = nes.read8(addr as u16);
        }

        nes.write8(addr, value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::ZpgX,
            args: vec![lo],
            value: AddressingResult::AbsZpgInfo(addr, old),
        }
    }
}

pub fn ZpgY_store<'a>(
    nes: &'a Nes,
    op: impl RegOperations + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let lo = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let addr = (lo as u16 + nes.cpu.y.get() as u16) & 0x00FF;
        let value = op.read(nes);
        yield CpuStep::Cycle;
        let mut old = 0;

        if cfg!(feature = "nestest") {
            old = nes.read8(addr as u16);
        }

        nes.write8(addr, value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::ZpgY,
            args: vec![lo],
            value: AddressingResult::AbsZpgInfo(addr, old),
        }
    }
}

pub fn AbsY_store<'a>(
    nes: &'a Nes,
    op: impl RegOperations + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let lo = Cpu::fetch_byte_inc_pc(&nes) as u16;
        yield CpuStep::Cycle;

        let hi = Cpu::fetch_byte_inc_pc(&nes) as u16;
        yield CpuStep::Cycle;

        let y = nes.cpu.y.get() as u16;
        let tmp_addr = lo | hi << 8;
        let effective_addr = (tmp_addr as u32 + y as u32) as u16;
        yield CpuStep::Cycle;

        let mut old = 0;
        if cfg!(feature = "nestest") {
            old = nes.read8(effective_addr as u16);
        }

        let value = op.read(nes);
        nes.write8(effective_addr, value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::AbsY,
            args: vec![lo as u8, hi as u8],
            value: AddressingResult::AbsZpgInfo(effective_addr, old),
        }
    }
}

pub fn AbsX_store<'a>(
    nes: &'a Nes,
    op: impl RegOperations + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let lo = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let hi = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let x = nes.cpu.x.get();
        let addr = ((lo as u16) | ((hi as u16) << 8)) + (x as u16);
        yield CpuStep::Cycle;

        let value = op.read(nes);
        let mut old = 0;

        if cfg!(feature = "nestest") {
            old = nes.read8(addr as u16);
        }

        nes.write8(addr, value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::AbsX,
            args: vec![lo, hi],
            value: AddressingResult::AbsZpgInfo(addr, old),
        }
    }
}

//no Imm_store

pub fn Transf<'a>(
    nes: &'a Nes,
    src: impl RegOperations + 'a,
    dst: impl RegOperations + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;
        let new_value = src.read(nes);

        dst.load(nes, new_value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::Implied,
            args: vec![],
            value: AddressingResult::None,
        }
    }
}

pub fn Impl_calc<'a>(
    nes: &'a Nes,
    op: impl ImplOperation + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        op.calc_and_set(nes);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::Implied,
            args: vec![],
            value: AddressingResult::None,
        }
    }
}

pub fn Impl_calc3<'a>(
    nes: &'a Nes,
    op: impl ImplOperation + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        op.calc_and_set(nes);
        yield CpuStep::Cycle;
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::Implied,
            args: vec![],
            value: AddressingResult::None,
        }
    }
}

pub fn Impl_calc4<'a>(
    nes: &'a Nes,
    op: impl ImplOperation + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        op.calc_and_set(nes);
        yield CpuStep::Cycle;
        yield CpuStep::Cycle;
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::Implied,
            args: vec![],
            value: AddressingResult::None,
        }
    }
}

pub fn Xind_calc<'a>(
    nes: &'a Nes,
    op: impl BinaryOperation + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let base_addr = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let x = nes.cpu.x.get();
        let target_addr = base_addr + x;
        yield CpuStep::Cycle;

        let lo = nes.read8(target_addr as u16);
        yield CpuStep::Cycle;

        let hi = nes.read8((target_addr + 1) as u16);
        yield CpuStep::Cycle;

        let addr = (lo as u16) | ((hi as u16) << 8);
        let value = nes.read8(addr);
        op.calc_and_set(nes, value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::XInd,
            args: vec![base_addr],
            value: AddressingResult::IndInfo(base_addr, target_addr as u16, addr, value),
        }
    }
}

pub fn Zpg_calc<'a>(
    nes: &'a Nes,
    op: impl BinaryOperation + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let addr = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let value = nes.read8(addr as u16);
        op.calc_and_set(nes, value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::Zpg,
            args: vec![addr],
            value: AddressingResult::RWValue(value),
        }
    }
}

pub fn Imm_calc<'a>(
    nes: &'a Nes,
    op: impl BinaryOperation + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let value = Cpu::fetch_byte_inc_pc(&nes);
        op.calc_and_set(nes, value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::Implied,
            args: vec![value],
            value: AddressingResult::RWValue(value),
        }
    }
}

pub fn Abs_calc<'a>(
    nes: &'a Nes,
    op: impl BinaryOperation + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let lo = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let hi = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let addr = (lo as u16) | ((hi as u16) << 8);
        let value = nes.read8(addr);
        op.calc_and_set(nes, value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::Abs,
            args: vec![lo, hi],
            value: AddressingResult::RWValue(value),
        }
    }
}

pub fn IndY_calc<'a>(
    nes: &'a Nes,
    op: impl BinaryOperation + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let base_addr = Cpu::fetch_byte_inc_pc(&nes);
        let base_addr_lo = base_addr as u16;
        let base_addr_hi = (base_addr_lo + 1) & 0x00FF;
        yield CpuStep::Cycle;

        let lo = nes.read8(base_addr_lo) as u16;
        yield CpuStep::Cycle;

        let hi = nes.read8(base_addr_hi) as u16;
        yield CpuStep::Cycle;

        let y = nes.cpu.y.get() as u16;
        let tmp_addr = lo | hi << 8;
        let addr = ((lo + y) & 0x00FF) | (hi << 8);
        let effective_addr = (tmp_addr as u32 + y as u32) as u16;

        let addr = if addr == effective_addr {
            addr
        } else {
            yield CpuStep::Cycle;
            effective_addr
        };

        let new_value = nes.read8(addr);
        op.calc_and_set(nes, new_value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::IndY,
            args: vec![base_addr],
            value: AddressingResult::IndInfo(base_addr, tmp_addr, effective_addr, new_value),
        }
    }
}

pub fn ZpgX_calc<'a>(
    nes: &'a Nes,
    op: impl BinaryOperation + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let lo = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let addr = (lo as u16 + nes.cpu.x.get() as u16) & 0x00FF;
        let value = nes.read8(addr);
        yield CpuStep::Cycle;

        op.calc_and_set(nes, value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::ZpgX,
            args: vec![lo],
            value: AddressingResult::AbsZpgInfo(addr, value),
        }
    }
}

//not used
pub fn _ZpgY_calc<'a>(
    nes: &'a Nes,
    op: impl BinaryOperation + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let lo = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let addr = (lo + nes.cpu.y.get()) as u16;
        let value = nes.read8(addr);
        yield CpuStep::Cycle;

        op.calc_and_set(nes, value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::ZpgY,
            args: vec![lo],
            value: AddressingResult::AbsZpgInfo(addr, value),
        }
    }
}

pub fn AbsY_calc<'a>(
    nes: &'a Nes,
    op: impl BinaryOperation + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let lo = Cpu::fetch_byte_inc_pc(&nes) as u16;
        yield CpuStep::Cycle;

        let hi = Cpu::fetch_byte_inc_pc(&nes) as u16;
        yield CpuStep::Cycle;

        let y = nes.cpu.y.get() as u16;
        let tmp_addr = lo | hi << 8;
        let addr = (lo + y) | (hi << 8);
        let effective_addr = (tmp_addr as u32 + y as u32) as u16;

        let addr = if addr == effective_addr {
            addr
        } else {
            yield CpuStep::Cycle;
            effective_addr
        };

        let value = nes.read8(addr);
        op.calc_and_set(nes, value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::AbsY,
            args: vec![lo as u8, hi as u8],
            value: AddressingResult::AbsZpgInfo(addr, value),
        }
    }
}

pub fn AbsX_calc<'a>(
    nes: &'a Nes,
    op: impl BinaryOperation + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let lo = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let hi = Cpu::fetch_byte_inc_pc(&nes);
        let x = nes.cpu.x.get();
        let addr = ((lo as u16) | ((hi as u16) << 8)) + (x as u16);
        yield CpuStep::Cycle;

        let value = nes.read8(addr);
        op.calc_and_set(nes, value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::AbsX,
            args: vec![lo, hi],
            value: AddressingResult::AbsZpgInfo(addr, value),
        }
    }
}

//for ASL A, ROL A, LSR A, ROR A
pub fn Reg_calc<'a>(
    nes: &'a Nes,
    op: impl BinaryOperation + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let value = nes.cpu.a.get();
        op.calc_and_set(nes, value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::Implied,
            args: vec![],
            value: AddressingResult::RegA,
        }
    }
}

pub fn Mem_Zpg_calc<'a>(
    nes: &'a Nes,
    op: impl MemCalcOperation + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let addr = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let value = nes.read8(addr as u16);
        yield CpuStep::Cycle;

        let new_value = op.calc(nes, value);
        yield CpuStep::Cycle;

        nes.write8(addr as u16, new_value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::Zpg,
            args: vec![addr],
            value: AddressingResult::RWValue(value),
        }
    }
}

pub fn Mem_Abs_calc<'a>(
    nes: &'a Nes,
    op: impl MemCalcOperation + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let lo = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let hi = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let addr = (lo as u16) | ((hi as u16) << 8);
        let value = nes.read8(addr);
        yield CpuStep::Cycle;

        let new_value = op.calc(nes, value);
        yield CpuStep::Cycle;

        nes.write8(addr, new_value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::Abs,
            args: vec![lo, hi],
            value: AddressingResult::RWValue(value),
        }
    }
}

pub fn Mem_ZpgX_calc<'a>(
    nes: &'a Nes,
    op: impl MemCalcOperation + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let lo = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let addr = (lo as u16 + nes.cpu.x.get() as u16) & 0x00FF;
        yield CpuStep::Cycle;

        let value = nes.read8(addr);
        yield CpuStep::Cycle;

        let new_value = op.calc(nes, value);
        yield CpuStep::Cycle;

        nes.write8(addr, new_value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::ZpgX,
            args: vec![lo],
            value: AddressingResult::AbsZpgInfo(addr, value),
        }
    }
}

pub fn Mem_AbsX_calc<'a>(
    nes: &'a Nes,
    op: impl MemCalcOperation + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let lo = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let hi = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let x = nes.cpu.x.get();
        let addr = ((lo as u16) | ((hi as u16) << 8)) + (x as u16);
        yield CpuStep::Cycle;

        let value = nes.read8(addr);
        yield CpuStep::Cycle;

        let new_value = op.calc(nes, value);
        yield CpuStep::Cycle;

        nes.write8(addr, new_value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::AbsX,
            args: vec![lo, hi],
            value: AddressingResult::AbsZpgInfo(addr, value),
        }
    }
}

pub fn rel_branch<'a>(
    nes: &'a Nes,
    op: impl BranchOperation + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let offset = Cpu::fetch_byte_inc_pc(&nes) as i8;
        yield CpuStep::Cycle;

        let pc = nes.cpu.pc.get();

        let addr = (pc as i16 + offset as i16) as u16;

        if op.branch(nes) {
            if (pc ^ addr) & 0x0100 != 0 {
                yield CpuStep::Cycle;
            }

            nes.cpu.pc.set(addr);
            yield CpuStep::Cycle;
        }

        OpArg {
            addressing: Addressing::Branch,
            args: vec![offset as u8],
            value: AddressingResult::Address(addr),
        }
    }
}

pub fn abs_branch<'a>(nes: &'a Nes) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let lo = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let hi = Cpu::fetch_byte_inc_pc(&nes);
        let pc = (lo as u16) | ((hi as u16) << 8);
        nes.cpu.pc.set(pc);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::Branch,
            args: vec![lo, hi],
            value: AddressingResult::Address(pc),
        }
    }
}

pub fn ind_branch<'a>(nes: &'a Nes) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let base_lo = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let base_hi = Cpu::fetch_byte_inc_pc(&nes);
        let ptr_to_lo = base_lo as u16 | ((base_hi as u16) << 8);
        let ptr_to_hi = ((base_lo as u16 + 1) & 0x00FF) | ((base_hi as u16) << 8);
        yield CpuStep::Cycle;

        let lo = nes.read8(ptr_to_lo) as u16;
        yield CpuStep::Cycle;

        let hi = nes.read8(ptr_to_hi) as u16;
        let addr = lo | (hi << 8);
        nes.cpu.pc.set(addr);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::Branch,
            args: vec![base_lo, base_hi],
            value: AddressingResult::IndInfo(0, ptr_to_lo, addr, 0),
        }
    }
}

pub fn abs_test<'a>(
    nes: &'a Nes,
    op: impl TestOperation + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let lo = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let hi = Cpu::fetch_byte_inc_pc(&nes);
        let addr = (lo as u16) | ((hi as u16) << 8);
        yield CpuStep::Cycle;

        let m = nes.read8(addr);
        op.test(nes, m);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::Abs,
            args: vec![lo, hi],
            value: AddressingResult::RWValue(m),
        }
    }
}

pub fn zpg_test<'a>(
    nes: &'a Nes,
    op: impl TestOperation + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let addr = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let m = nes.read8(addr as u16);
        op.test(nes, m);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::Zpg,
            args: vec![addr],
            value: AddressingResult::RWValue(m),
        }
    }
}

pub fn rti<'a>(nes: &'a Nes) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let p = Cpu::pop(nes);
        yield CpuStep::Cycle;

        let pc_lo = Cpu::pop(nes);
        yield CpuStep::Cycle;

        let pc_hi = Cpu::pop(nes);
        yield CpuStep::Cycle;

        nes.cpu.p.set(Status::from_bits_truncate(p) | Status::U);
        yield CpuStep::Cycle;

        let pc = (pc_lo as u16) | ((pc_hi as u16) << 8);
        nes.cpu.pc.set(pc);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::Implied,
            args: vec![],
            value: AddressingResult::None,
        }
    }
}

pub fn rts<'a>(nes: &'a Nes) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        yield CpuStep::Cycle;

        let pc_lo = Cpu::pop(nes);
        yield CpuStep::Cycle;

        let pc_hi = Cpu::pop(nes);
        let pc = (pc_lo as u16) | ((pc_hi as u16) << 8);
        yield CpuStep::Cycle;

        nes.cpu.pc.set(pc + 1);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::Implied,
            args: vec![],
            value: AddressingResult::None,
        }
    }
}

pub fn jsr<'a>(nes: &'a Nes) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let pc_lo = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let pc_hi = Cpu::fetch_byte_inc_pc(&nes);
        let pc = (pc_lo as u16) | ((pc_hi as u16) << 8);
        yield CpuStep::Cycle;

        let return_addr = nes.cpu.pc.get() - 1;
        yield CpuStep::Cycle;

        let hi = ((return_addr & 0xFF00) >> 8) as u8;
        Cpu::push(nes, hi);
        yield CpuStep::Cycle;

        let lo = (return_addr & 0x00FF) as u8;
        Cpu::push(nes, lo);
        nes.cpu.pc.set(pc);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::Implied,
            args: vec![pc_lo, pc_hi],
            value: AddressingResult::Address(pc),
        }
    }
}

pub fn brk<'a>(nes: &'a Nes) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(nes);
        yield CpuStep::Cycle;

        Cpu::fetch_byte_inc_pc(nes);
        yield CpuStep::Cycle;

        let return_addr = nes.cpu.pc.get();
        let hi = ((return_addr & 0xFF00) >> 8) as u8;
        Cpu::push(nes, hi);
        yield CpuStep::Cycle;

        let lo = (return_addr & 0x00FF) as u8;
        Cpu::push(nes, lo);
        yield CpuStep::Cycle;

        let p = (nes.cpu.p.get() | Status::B | Status::U).bits();

        Cpu::push(nes, p);
        yield CpuStep::Cycle;

        let pc_hi = nes.read8(0xFFFE);
        yield CpuStep::Cycle;

        let pc_lo = nes.read8(0xFFFF);
        let pc = (pc_lo as u16) | ((pc_hi as u16) << 8);
        nes.cpu.pc.set(pc);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::Implied,
            args: vec![],
            value: AddressingResult::None,
        }
    }
}

////For Undocumented Instructions
pub fn Mem_AbsY_calc<'a>(
    nes: &'a Nes,
    op: impl MemCalcOperation + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let lo = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let hi = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let y = nes.cpu.y.get();
        let addr = ((lo as u16) | ((hi as u16) << 8)) + (y as u16);
        yield CpuStep::Cycle;

        let value = nes.read8(addr);
        yield CpuStep::Cycle;

        let new_value = op.calc(nes, value);
        yield CpuStep::Cycle;

        nes.write8(addr, new_value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::AbsY,
            args: vec![lo, hi],
            value: AddressingResult::AbsZpgInfo(addr, value),
        }
    }
}

pub fn Mem_Xind_calc<'a>(
    nes: &'a Nes,
    op: impl MemCalcOperation + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let base_addr = Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let x = nes.cpu.x.get();
        let target_addr = base_addr + x;
        yield CpuStep::Cycle;

        let lo = nes.read8(target_addr as u16);
        yield CpuStep::Cycle;

        let hi = nes.read8((target_addr + 1) as u16);
        yield CpuStep::Cycle;

        let addr = (lo as u16) | ((hi as u16) << 8);
        let value = nes.read8(addr);
        yield CpuStep::Cycle;

        let new_value = op.calc(nes, value);
        yield CpuStep::Cycle;

        nes.write8(addr, new_value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::XInd,
            args: vec![base_addr],
            value: AddressingResult::IndInfo(base_addr, target_addr as u16, addr, value),
        }
    }
}

pub fn Mem_IndY_calc<'a>(
    nes: &'a Nes,
    op: impl MemCalcOperation + 'a,
) -> impl Coroutine<Yield = CpuStep, Return = OpArg> + 'a {
    #[coroutine] move || {
        Cpu::fetch_byte_inc_pc(&nes);
        yield CpuStep::Cycle;

        let base_addr = Cpu::fetch_byte_inc_pc(&nes);
        let base_addr_lo = base_addr as u16;
        let base_addr_hi = (base_addr_lo + 1) & 0x00FF;
        yield CpuStep::Cycle;

        let lo = nes.read8(base_addr_lo) as u16;
        yield CpuStep::Cycle;

        let hi = nes.read8(base_addr_hi) as u16;
        yield CpuStep::Cycle;

        let y = nes.cpu.y.get() as u16;
        yield CpuStep::Cycle;
        let tmp_addr = lo | hi << 8;
        let addr = ((lo + y) & 0x00FF) | (hi << 8);
        let effective_addr = (tmp_addr as u32 + y as u32) as u16;

        let addr = if addr == effective_addr {
            addr
        } else {
            effective_addr
        };
        yield CpuStep::Cycle;

        let value = nes.read8(addr);
        let new_value = op.calc(nes, value);
        yield CpuStep::Cycle;

        nes.write8(addr, new_value);
        yield CpuStep::Cycle;

        OpArg {
            addressing: Addressing::IndY,
            args: vec![base_addr],
            value: AddressingResult::IndInfo(base_addr, tmp_addr, effective_addr, value),
        }
    }
}
