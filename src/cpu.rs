use bitflags::bitflags;
use std::cell::Cell;
use std::fmt;
use std::ops::{Coroutine, CoroutineState};

use crate::opcode::Opcode;

use super::addressings::*;
use super::opcode;
use super::operations::*;
use super::Nes;

use super::yield_all;

#[derive(Default, Clone)]
pub struct Cpu {
    pub pc: Cell<u16>,
    pub a: Cell<u8>,
    pub x: Cell<u8>,
    pub y: Cell<u8>,
    pub s: Cell<u8>,
    pub p: Cell<Status>,
    pub nmi: Cell<bool>,
}

#[derive(Debug, Clone)]
pub enum CpuStep {
    Cycle,
    Op(u16, opcode::Opcode, OpArg),
    DebugLine(String),
}

pub fn debug_reg(cpu: &Cpu) -> String {
    let a = format!("A:{:02X}", cpu.a.get());
    let x = format!("X:{:02X}", cpu.x.get());
    let y = format!("Y:{:02X}", cpu.y.get());
    let p = format!("P:{:02X}", cpu.p.get());
    let sp = format!("SP:{:02X}", cpu.s.get());

    format!("{} {} {} {} {}", a, x, y, p, sp)
}

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq)]
    pub struct Status : u8 {
        //carry
        const C = 1 << 0;
        //zero
        const Z = 1 <<1;
        //Interript EN
        const I = 1 <<2;
        //Decimal mode
        const D = 1 << 3;
        //Break flag
        const B = 1 << 4;
        //Unused (always 1)
        const U = 1 << 5;
        //Signed Overflow
        const V = 1 << 6;
        //Negative
        const N = 1 << 7;
    }
}

impl Cpu {
    pub fn run<'a>(nes: &'a Nes) -> impl Coroutine<Yield = CpuStep, Return = !> + 'a {
        #[coroutine] move || loop {
            if nes.cpu.nmi.get() {
                nes.cpu.nmi.set(false);
                let ret_addr = nes.cpu.pc.get();
                let hi = ((ret_addr & 0xFF00) >> 8) as u8;
                let lo = (ret_addr & 0x00FF) as u8;
                Cpu::push(nes, hi);
                Cpu::push(nes, lo);
                Cpu::push(nes, nes.cpu.p.get().bits());
                nes.cpu.p.update(|p| p | Status::I);
                let nmi_vector = nes.read16(0xFFFA);
                nes.cpu.pc.set(nmi_vector);
            }

            //Not consume Cpu Cycle
            let pc = nes.cpu.pc.get();
            let bin = nes.read8(pc);
            let opcode: Opcode = num_traits::FromPrimitive::from_u8(bin)
                .unwrap_or_else(|| panic!("Unknown Opcode 0x{:02X} @0x{:04X}", bin, pc));

            let oparg = match opcode {
                ////////////////////
                // LDA Operations
                ////////////////////
                Opcode::LdaXind => {
                    yield_all! { Xind_load(nes, RegA) }
                }
                Opcode::LdaZpg => {
                    yield_all! { Zpg_load(nes, RegA) }
                }
                Opcode::LdaAbs => {
                    yield_all! { Abs_load(nes, RegA) }
                }
                Opcode::LdaIndY => {
                    yield_all! { IndY_load(nes, RegA) }
                }
                Opcode::LdaZpgX => {
                    yield_all! { ZpgX_load(nes, RegA) }
                }
                Opcode::LdaAbsY => {
                    yield_all! { AbsY_load(nes, RegA) }
                }
                Opcode::LdaAbsX => {
                    yield_all! { AbsX_load(nes, RegA) }
                }
                Opcode::LdaImm => {
                    yield_all! { Imm_load(nes, RegA) }
                }

                ////////////////////
                // LDY Operations
                ////////////////////
                Opcode::LdyImm => {
                    yield_all! { Imm_load(nes, RegY) }
                }
                Opcode::LdyZpg => {
                    yield_all! { Zpg_load(nes, RegY) }
                }
                Opcode::LdyAbs => {
                    yield_all! { Abs_load(nes, RegY) }
                }
                Opcode::LdyZpgX => {
                    yield_all! { ZpgX_load(nes, RegY) }
                }
                Opcode::LdyAbsX => {
                    yield_all! { AbsX_load(nes, RegY) }
                }

                ////////////////////
                // LDX Operations
                ////////////////////
                Opcode::LdxImm => {
                    yield_all! { Imm_load(nes, RegX) }
                }
                Opcode::LdxZpg => {
                    yield_all! { Zpg_load(nes, RegX) }
                }
                Opcode::LdxAbs => {
                    yield_all! { Abs_load(nes, RegX) }
                }
                Opcode::LdxZpgY => {
                    yield_all! { ZpgY_load(nes, RegX) }
                }
                Opcode::LdxAbsY => {
                    yield_all! { AbsY_load(nes, RegX) }
                }

                ////////////////////
                // STA Operations
                ////////////////////
                Opcode::StaXind => {
                    yield_all! { Xind_store(nes, RegA) }
                }
                Opcode::StaZpg => {
                    yield_all! { Zpg_store(nes, RegA) }
                }
                Opcode::StaAbs => {
                    yield_all! { Abs_store(nes, RegA) }
                }
                Opcode::StaIndY => {
                    yield_all! { IndY_store(nes, RegA) }
                }
                Opcode::StaZpgX => {
                    yield_all! { ZpgX_store(nes, RegA) }
                }
                Opcode::StaAbsY => {
                    yield_all! { AbsY_store(nes, RegA) }
                }
                Opcode::StaAbsX => {
                    yield_all! { AbsX_store(nes, RegA) }
                }

                ////////////////////
                // STY Operations
                ////////////////////
                Opcode::StyZpg => {
                    yield_all! { Zpg_store(nes, RegY) }
                }
                Opcode::StyAbs => {
                    yield_all! { Abs_store(nes, RegY) }
                }
                Opcode::StyZpgX => {
                    yield_all! { ZpgX_store(nes, RegY) }
                }

                ////////////////////
                // STX Operations
                ////////////////////
                Opcode::StxZpg => {
                    yield_all! { Zpg_store(nes, RegX) }
                }
                Opcode::StxAbs => {
                    yield_all! { Abs_store(nes, RegX) }
                }
                Opcode::StxZpgY => {
                    yield_all! { ZpgY_store(nes, RegX) }
                }

                ////////////////////
                // ORA Operations
                ////////////////////
                Opcode::OraXind => {
                    yield_all! { Xind_calc(nes, ORA) }
                }
                Opcode::OraZpg => {
                    yield_all! { Zpg_calc(nes, ORA) }
                }
                Opcode::OraImm => {
                    yield_all! { Imm_calc(nes, ORA) }
                }
                Opcode::OraAbs => {
                    yield_all! { Abs_calc(nes, ORA) }
                }
                Opcode::OraIndY => {
                    yield_all! { IndY_calc(nes, ORA) }
                }
                Opcode::OraZpgX => {
                    yield_all! { ZpgX_calc(nes, ORA) }
                }
                Opcode::OraAbsY => {
                    yield_all! { AbsY_calc(nes, ORA) }
                }
                Opcode::OraAbsX => {
                    yield_all! { AbsX_calc(nes, ORA) }
                }

                ////////////////////
                // AND Operations
                ////////////////////
                Opcode::AndXind => {
                    yield_all! { Xind_calc(nes, AND) }
                }
                Opcode::AndZpg => {
                    yield_all! { Zpg_calc(nes, AND) }
                }
                Opcode::AndImm => {
                    yield_all! { Imm_calc(nes, AND) }
                }
                Opcode::AndAbs => {
                    yield_all! { Abs_calc(nes, AND) }
                }
                Opcode::AndIndY => {
                    yield_all! { IndY_calc(nes, AND) }
                }
                Opcode::AndZpgX => {
                    yield_all! { ZpgX_calc(nes, AND) }
                }
                Opcode::AndAbsY => {
                    yield_all! { AbsY_calc(nes, AND) }
                }
                Opcode::AndAbsX => {
                    yield_all! { AbsX_calc(nes, AND) }
                }

                ////////////////////
                // EOR Operations
                ////////////////////
                Opcode::EorXind => {
                    yield_all! { Xind_calc(nes, EOR) }
                }
                Opcode::EorZpg => {
                    yield_all! { Zpg_calc(nes, EOR) }
                }
                Opcode::EorImm => {
                    yield_all! { Imm_calc(nes, EOR) }
                }
                Opcode::EorAbs => {
                    yield_all! { Abs_calc(nes, EOR) }
                }
                Opcode::EorIndY => {
                    yield_all! { IndY_calc(nes, EOR) }
                }
                Opcode::EorZpgX => {
                    yield_all! {ZpgX_calc(nes, EOR) }
                }
                Opcode::EorAbsY => {
                    yield_all! { AbsY_calc(nes, EOR) }
                }
                Opcode::EorAbsX => {
                    yield_all! { AbsX_calc(nes, EOR) }
                }

                ////////////////////
                // ADC Operations
                ////////////////////
                Opcode::AdcXind => {
                    yield_all! { Xind_calc(nes, ADC) }
                }
                Opcode::AdcZpg => {
                    yield_all! { Zpg_calc(nes, ADC) }
                }
                Opcode::AdcImm => {
                    yield_all! { Imm_calc(nes, ADC) }
                }
                Opcode::AdcAbs => {
                    yield_all! { Abs_calc(nes, ADC) }
                }
                Opcode::AdcIndY => {
                    yield_all! { IndY_calc(nes, ADC) }
                }
                Opcode::AdcZpgX => {
                    yield_all! { ZpgX_calc(nes, ADC) }
                }
                Opcode::AdcAbsY => {
                    yield_all! { AbsY_calc(nes, ADC) }
                }
                Opcode::AdcAbsX => {
                    yield_all! { AbsX_calc(nes, ADC) }
                }

                ////////////////////
                // SBC Operations
                ////////////////////
                Opcode::SbcXind => {
                    yield_all! { Xind_calc(nes, SBC) }
                }
                Opcode::SbcZpg => {
                    yield_all! { Zpg_calc(nes, SBC) }
                }
                Opcode::SbcImm => {
                    yield_all! { Imm_calc(nes, SBC) }
                }
                Opcode::SbcAbs => {
                    yield_all! { Abs_calc(nes, SBC) }
                }
                Opcode::SbcIndY => {
                    yield_all! { IndY_calc(nes, SBC) }
                }
                Opcode::SbcZpgX => {
                    yield_all! { ZpgX_calc(nes, SBC) }
                }
                Opcode::SbcAbsY => {
                    yield_all! { AbsY_calc(nes, SBC) }
                }
                Opcode::SbcAbsX => {
                    yield_all! { AbsX_calc(nes, SBC) }
                }

                ////////////////////
                // ASL Operations
                ////////////////////
                Opcode::AslZpg => {
                    yield_all! { Mem_Zpg_calc(nes, ASL) }
                }
                Opcode::AslAbs => {
                    yield_all! { Mem_Abs_calc(nes, ASL) }
                }
                Opcode::AslZpgX => {
                    yield_all! { Mem_ZpgX_calc(nes, ASL) }
                }
                Opcode::AslAbsX => {
                    yield_all! { Mem_AbsX_calc(nes, ASL) }
                }
                Opcode::AslA => {
                    yield_all! { Reg_calc(nes, ASL) }
                }

                ////////////////////
                // Rol Operations
                ////////////////////
                Opcode::RolZpg => {
                    yield_all! { Mem_Zpg_calc(nes, ROL) }
                }
                Opcode::RolAbs => {
                    yield_all! { Mem_Abs_calc(nes, ROL) }
                }
                Opcode::RolZpgX => {
                    yield_all! { Mem_ZpgX_calc(nes, ROL) }
                }
                Opcode::RolAbsX => {
                    yield_all! { Mem_AbsX_calc(nes, ROL) }
                }
                Opcode::RolA => {
                    yield_all! { Reg_calc(nes, ROL) }
                }

                ////////////////////
                // Lsr Operations
                ////////////////////
                Opcode::LsrZpg => {
                    yield_all! { Mem_Zpg_calc(nes, LSR) }
                }
                Opcode::LsrAbs => {
                    yield_all! { Mem_Abs_calc(nes, LSR) }
                }
                Opcode::LsrZpgX => {
                    yield_all! { Mem_ZpgX_calc(nes, LSR) }
                }
                Opcode::LsrAbsX => {
                    yield_all! { Mem_AbsX_calc(nes, LSR) }
                }
                Opcode::LsrA => {
                    yield_all! { Reg_calc(nes, LSR) }
                }

                ////////////////////
                // Ror Operations
                ////////////////////
                Opcode::RorZpg => {
                    yield_all! { Mem_Zpg_calc(nes, ROR) }
                }
                Opcode::RorAbs => {
                    yield_all! { Mem_Abs_calc(nes, ROR) }
                }
                Opcode::RorZpgX => {
                    yield_all! { Mem_ZpgX_calc(nes, ROR) }
                }
                Opcode::RorAbsX => {
                    yield_all! { Mem_AbsX_calc(nes, ROR) }
                }
                Opcode::RorA => {
                    yield_all! { Reg_calc(nes, ROR) }
                }

                ////////////////////
                // CMP Operations
                ////////////////////
                Opcode::CmpXind => {
                    yield_all! { Xind_calc(nes, CMP) }
                }
                Opcode::CmpZpg => {
                    yield_all! { Zpg_calc(nes, CMP) }
                }
                Opcode::CmpImm => {
                    yield_all! { Imm_calc(nes, CMP) }
                }
                Opcode::CmpAbs => {
                    yield_all! { Abs_calc(nes, CMP) }
                }
                Opcode::CmpIndY => {
                    yield_all! { IndY_calc(nes, CMP) }
                }
                Opcode::CmpZpgX => {
                    yield_all! { ZpgX_calc(nes, CMP) }
                }
                Opcode::CmpAbsY => {
                    yield_all! { AbsY_calc(nes, CMP) }
                }
                Opcode::CmpAbsX => {
                    yield_all! { AbsX_calc(nes, CMP) }
                }

                ////////////////////
                // CPY Operations
                ////////////////////
                Opcode::CpyZpg => {
                    yield_all! { Zpg_calc(nes, CPY) }
                }
                Opcode::CpyImm => {
                    yield_all! { Imm_calc(nes, CPY) }
                }
                Opcode::CpyAbs => {
                    yield_all! { Abs_calc(nes, CPY) }
                }

                ////////////////////
                // CPX Operations
                ////////////////////
                Opcode::CpxZpg => {
                    yield_all! { Zpg_calc(nes, CPX) }
                }
                Opcode::CpxImm => {
                    yield_all! { Imm_calc(nes, CPX) }
                }
                Opcode::CpxAbs => {
                    yield_all! { Abs_calc(nes, CPX) }
                }

                ////////////////////
                // Transfer Operations
                ////////////////////
                Opcode::TaxImpl => {
                    // a -> x
                    yield_all! { Transf(nes, RegA, RegX) }
                }
                Opcode::TxaImpl => {
                    yield_all! { Transf(nes, RegX, RegA) }
                }
                Opcode::TxsImpl => {
                    yield_all! { Transf(nes, RegX, RegS) }
                }
                Opcode::TsxImpl => {
                    yield_all! { Transf(nes, RegS, RegX) }
                }
                Opcode::TayImpl => {
                    yield_all! { Transf(nes, RegA, RegY) }
                }
                Opcode::TyaImpl => {
                    yield_all! { Transf(nes, RegY, RegA) }
                }

                ////////////////////
                // Stack Operations
                ////////////////////
                Opcode::PhpImpl => {
                    yield_all! { Impl_calc3(nes, PHP) } //3
                }
                Opcode::PlpImpl => {
                    yield_all! { Impl_calc4(nes, PLP) } //4
                }
                Opcode::PhaImpl => {
                    yield_all! { Impl_calc3(nes, PHA) } //3
                }
                Opcode::PlaImpl => {
                    yield_all! { Impl_calc4(nes, PLA) } //4
                }

                ////////////////////
                // FlagReg Operations
                ////////////////////
                Opcode::ClcImpl => {
                    yield_all! { Impl_calc(nes, CLC) } //2
                }
                Opcode::SecImpl => {
                    yield_all! { Impl_calc(nes, SEC) } //2
                }
                Opcode::CliImpl => {
                    yield_all! { Impl_calc(nes, CLI) } //2
                }
                Opcode::SeiImpl => {
                    yield_all! { Impl_calc(nes, SEI) } //2
                }
                Opcode::ClvImpl => {
                    yield_all! { Impl_calc(nes, CLV) } //2
                }
                Opcode::CldImpl => {
                    yield_all! { Impl_calc(nes, CLD) } //2
                }
                Opcode::SedImpl => {
                    yield_all! { Impl_calc(nes, SED) } //2
                }

                ////////////////////
                // INC/DEC Operations
                ////////////////////
                Opcode::DeyImpl => {
                    yield_all! { Impl_calc(nes, DEY) } //2
                }
                Opcode::InyImpl => {
                    yield_all! { Impl_calc(nes, INY) } //2
                }
                Opcode::DexImpl => {
                    yield_all! { Impl_calc(nes, DEX) } //2
                }
                Opcode::InxImpl => {
                    yield_all! { Impl_calc(nes, INX) } //2
                }
                Opcode::DecZpg => {
                    yield_all! { Mem_Zpg_calc(nes, DEC) }
                }
                Opcode::DecAbs => {
                    yield_all! { Mem_Abs_calc(nes, DEC) }
                }
                Opcode::DecZpgX => {
                    yield_all! { Mem_ZpgX_calc(nes, DEC) }
                }
                Opcode::DecAbsX => {
                    yield_all! { Mem_AbsX_calc(nes, DEC) }
                }
                Opcode::IncZpg => {
                    yield_all! { Mem_Zpg_calc(nes, INC) }
                }
                Opcode::IncAbs => {
                    yield_all! { Mem_Abs_calc(nes, INC) }
                }
                Opcode::IncZpgX => {
                    yield_all! { Mem_ZpgX_calc(nes, INC) }
                }
                Opcode::IncAbsX => {
                    yield_all! { Mem_AbsX_calc(nes, INC) }
                }

                ////////////////////
                // NOP Operations
                ////////////////////
                Opcode::NopImpl => {
                    yield_all! { Impl_calc(nes, NOP) }
                }

                ////////////////////
                // Branch Operations
                ////////////////////
                Opcode::BplRel => {
                    yield_all! { rel_branch(nes, BPL) }
                }
                Opcode::BmiRel => {
                    yield_all! { rel_branch(nes, BMI) }
                }
                Opcode::BvcRel => {
                    yield_all! { rel_branch(nes, BVC) }
                }
                Opcode::BvsRel => {
                    yield_all! { rel_branch(nes, BVS) }
                }
                Opcode::BccRel => {
                    yield_all! { rel_branch(nes, BCC) }
                }
                Opcode::BcsRel => {
                    yield_all! { rel_branch(nes, BCS) }
                }
                Opcode::BneRel => {
                    yield_all! { rel_branch(nes, BNE) }
                }
                Opcode::BeqRel => {
                    yield_all! { rel_branch(nes, BEQ) }
                }

                ////////////////////
                // Jmp Operations
                ////////////////////
                Opcode::JmpAbs => {
                    yield_all! { abs_branch(nes) }
                }
                Opcode::JmpInd => {
                    yield_all! { ind_branch(nes) }
                }

                ////////////////////
                // Test Operations
                ////////////////////
                Opcode::BitAbs => {
                    yield_all! { abs_test(nes, BIT) }
                }
                Opcode::BitZpg => {
                    yield_all! { zpg_test(nes, BIT) }
                }

                ////////////////////
                // Return Operation
                ////////////////////
                Opcode::RtiImpl => {
                    yield_all! { rti(nes) }
                }
                Opcode::RtsImpl => {
                    yield_all! { rts(nes) }
                }

                ////////////////////
                // Subrouting Operation
                ////////////////////
                Opcode::JsrAbs => {
                    yield_all! { jsr(nes) }
                }

                ////////////////////
                // Special Operation
                ////////////////////
                Opcode::BrkImpl => {
                    yield_all! { brk(nes) }
                }

                /////////////////
                // Undocumented Instructions
                ////////////////
                Opcode::UNopZpg1 | Opcode::UNopZpg2 | Opcode::UNopZpg3 => {
                    yield_all! { Zpg_load(nes, NOP) }
                }
                Opcode::UNopZpgX1
                | Opcode::UNopZpgX2
                | Opcode::UNopZpgX3
                | Opcode::UNopZpgX4
                | Opcode::UNopZpgX5
                | Opcode::UNopZpgX6 => {
                    yield_all! { ZpgX_load(nes, NOP) }
                }
                Opcode::UNopImm1
                | Opcode::UNopImm2
                | Opcode::UNopImm3
                | Opcode::UNopImm4
                | Opcode::UNopImm5 => {
                    yield_all! { Imm_load(nes, NOP) }
                }
                Opcode::UNopAbs => {
                    yield_all! { Abs_load(nes, NOP) }
                }
                Opcode::UNopAbsX1
                | Opcode::UNopAbsX2
                | Opcode::UNopAbsX3
                | Opcode::UNopAbsX4
                | Opcode::UNopAbsX5
                | Opcode::UNopAbsX6 => {
                    yield_all! { AbsX_load(nes, NOP) }
                }
                Opcode::UNopImpl1
                | Opcode::UNopImpl2
                | Opcode::UNopImpl3
                | Opcode::UNopImpl4
                | Opcode::UNopImpl5
                | Opcode::UNopImpl6 => {
                    yield_all! { Impl_calc(nes, NOP) }
                }
                Opcode::ULaxZpg => {
                    yield_all! { Zpg_load(nes, LAX) }
                }
                Opcode::ULaxZpgY => {
                    yield_all! { ZpgY_load(nes, LAX) }
                }
                Opcode::ULaxAbs => {
                    yield_all! { Abs_load(nes, LAX) }
                }
                Opcode::ULaxAbsY => {
                    yield_all! { AbsY_load(nes, LAX) }
                }
                Opcode::ULaxXind => {
                    yield_all! { Xind_load(nes, LAX) }
                }
                Opcode::ULaxIndY => {
                    yield_all! { IndY_load(nes, LAX) }
                }
                Opcode::USaxZpg => {
                    yield_all! { Zpg_store(nes, SAX) }
                }
                Opcode::USaxZpgY => {
                    yield_all! { ZpgY_store(nes, SAX) }
                }
                Opcode::USaxXind => {
                    yield_all! { Xind_store(nes, SAX) }
                }
                Opcode::USaxAbs => {
                    yield_all! { Abs_store(nes, SAX) }
                }
                Opcode::USbcImm => {
                    yield_all! { Imm_calc(nes, SBC) }
                }
                Opcode::UDcpZpg => {
                    yield_all! { Mem_Zpg_calc(nes, DCP) }
                }
                Opcode::UDcpZpgX => {
                    yield_all! { Mem_ZpgX_calc(nes, DCP)}
                }
                Opcode::UDcpAbs => {
                    yield_all! { Mem_Abs_calc(nes, DCP) }
                }
                Opcode::UDcpAbsX => {
                    yield_all! { Mem_AbsX_calc(nes, DCP) }
                }
                Opcode::UDcpAbsY => {
                    yield_all! { Mem_AbsY_calc(nes, DCP) }
                }
                Opcode::UDcpXind => {
                    yield_all! { Mem_Xind_calc(nes, DCP) }
                }
                Opcode::UDcpIndY => {
                    yield_all! { Mem_IndY_calc(nes, DCP) }
                }
                Opcode::UIsbZpg => {
                    yield_all! { Mem_Zpg_calc(nes, ISB) }
                }
                Opcode::UIsbZpgX => {
                    yield_all! { Mem_ZpgX_calc(nes, ISB) }
                }
                Opcode::UIsbAbs => {
                    yield_all! { Mem_Abs_calc(nes, ISB) }
                }
                Opcode::UIsbAbsX => {
                    yield_all! { Mem_AbsX_calc(nes, ISB) }
                }
                Opcode::UIsbAbsY => {
                    yield_all! { Mem_AbsY_calc(nes, ISB) }
                }
                Opcode::UIsbXind => {
                    yield_all! { Mem_Xind_calc(nes, ISB) }
                }
                Opcode::UIsbIndY => {
                    yield_all! { Mem_IndY_calc(nes, ISB) }
                }
                Opcode::USloZpg => {
                    yield_all! { Mem_Zpg_calc(nes, SLO) }
                }
                Opcode::USloZpgX => {
                    yield_all! { Mem_ZpgX_calc(nes, SLO) }
                }
                Opcode::USloAbs => {
                    yield_all! { Mem_Abs_calc(nes, SLO) }
                }
                Opcode::USloAbsX => {
                    yield_all! { Mem_AbsX_calc(nes, SLO) }
                }
                Opcode::USloAbsY => {
                    yield_all! { Mem_AbsY_calc(nes, SLO) }
                }
                Opcode::USloXind => {
                    yield_all! { Mem_Xind_calc(nes, SLO) }
                }
                Opcode::USloIndY => {
                    yield_all! { Mem_IndY_calc(nes, SLO) }
                }
                Opcode::URlaZpg => {
                    yield_all! { Mem_Zpg_calc(nes, RLA) }
                }
                Opcode::URlaZpgX => {
                    yield_all! { Mem_ZpgX_calc(nes, RLA) }
                }
                Opcode::URlaAbs => {
                    yield_all! { Mem_Abs_calc(nes, RLA) }
                }
                Opcode::URlaAbsX => {
                    yield_all! { Mem_AbsX_calc(nes, RLA) }
                }
                Opcode::URlaAbsY => {
                    yield_all! { Mem_AbsY_calc(nes, RLA) }
                }
                Opcode::URlaXind => {
                    yield_all! { Mem_Xind_calc(nes, RLA) }
                }
                Opcode::URlaIndY => {
                    yield_all! { Mem_IndY_calc(nes, RLA) }
                }
                Opcode::USreZpg => {
                    yield_all! { Mem_Zpg_calc(nes, SRE) }
                }
                Opcode::USreZpgX => {
                    yield_all! { Mem_ZpgX_calc(nes, SRE) }
                }
                Opcode::USreAbs => {
                    yield_all! { Mem_Abs_calc(nes, SRE) }
                }
                Opcode::USreAbsX => {
                    yield_all! { Mem_AbsX_calc(nes, SRE) }
                }
                Opcode::USreAbsY => {
                    yield_all! { Mem_AbsY_calc(nes, SRE) }
                }
                Opcode::USreXind => {
                    yield_all! { Mem_Xind_calc(nes, SRE) }
                }
                Opcode::USreIndY => {
                    yield_all! { Mem_IndY_calc(nes, SRE) }
                }
                Opcode::URraZpg => {
                    yield_all! { Mem_Zpg_calc(nes, RRA) }
                }
                Opcode::URraZpgX => {
                    yield_all! { Mem_ZpgX_calc(nes, RRA) }
                }
                Opcode::URraAbs => {
                    yield_all! { Mem_Abs_calc(nes, RRA) }
                }
                Opcode::URraAbsX => {
                    yield_all! { Mem_AbsX_calc(nes, RRA) }
                }
                Opcode::URraAbsY => {
                    yield_all! { Mem_AbsY_calc(nes, RRA) }
                }
                Opcode::URraXind => {
                    yield_all! { Mem_Xind_calc(nes, RRA) }
                }
                Opcode::URraIndY => {
                    yield_all! { Mem_IndY_calc(nes, RRA) }
                }
            };

            yield CpuStep::Op(pc, opcode, oparg);
        }
    }

    pub fn power_up() -> Self {
        Cpu {
            s: Cell::new(0xFD),
            p: Cell::new(Status::from_bits_truncate(0x34)),
            nmi: Cell::new(false),
            ..Cpu::default()
        }
    }

    pub fn fetch_byte_inc_pc(nes: &Nes) -> u8 {
        let pc = nes.cpu.pc.get();
        let ret = nes.read8(pc);
        nes.cpu.pc.set(pc + 1);
        ret
    }

    #[inline(always)]
    pub fn stack_addr(&self) -> u16 {
        self.s.get() as u16 | 0x0100
    }

    pub fn push(nes: &Nes, value: u8) {
        nes.write8(nes.cpu.stack_addr(), value);
        nes.cpu.s.update(|s| s - 1);
    }

    pub fn pop(nes: &Nes) -> u8 {
        nes.cpu.s.update(|s| s + 1);
        let res = nes.read8(nes.cpu.stack_addr());
        res
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pc = format!("pc : 0x{:04X}", self.pc.get());
        let a = format!("a : 0x{:02X}", self.a.get());
        let x = format!("x : 0x{:02X}", self.x.get());
        let y = format!("y : 0x{:02X}", self.y.get());
        let s = format!("s : 0x{:02X}", self.s.get());
        let p = format!("p : {:?}", self.p.get());
        let nmi = format!("nmi: {:?}", self.nmi.get());
        write!(
            f,
            "Cpu {{ {}, {}, {}, {}, {}, {}, {} }}",
            pc, a, x, y, s, p, nmi
        )
    }
}

pub fn debug_op(op: &CpuStep) -> String {
    let (pc, op, arg) = match op {
        CpuStep::Op(pc, op, args) => (pc, op, args),
        _ => unreachable!(),
    };

    let op_hex = *op as u8;
    let op_str = opcode::OpcodeStr(op);
    let arg_hex = arg
        .args
        .clone()
        .into_iter()
        .map(|hex| format!("{:02X}", hex))
        .collect::<Vec<_>>()
        .join(" ");

    let arg_mean = match (arg.addressing, arg.value) {
        //RWValue
        (Addressing::Abs, AddressingResult::RWValue(value)) => {
            format!("${:02X}{:02X} = {:02X}", arg.args[1], arg.args[0], value)
        }
        (Addressing::Zpg, AddressingResult::RWValue(value)) => {
            format!("${:02X} = {:02X}", arg.args[0], value)
        }
        (_, AddressingResult::RWValue(value)) => {
            format!("#${:02X}", value)
        }
        //Address
        (_, AddressingResult::Address(addr)) => format!("${:04X}", addr),
        //XInd, IndY, Branch
        (Addressing::XInd, AddressingResult::IndInfo(offset, base, addr, value)) => {
            format!(
                "(${:02X},X) @ {:02X} = {:04X} = {:02X}",
                offset, base, addr, value
            )
        }
        (Addressing::IndY, AddressingResult::IndInfo(offset, tmp_addr, effective_addr, value)) => {
            format!(
                "(${:02X}),Y = {:04X} @ {:04X} = {:02X}",
                offset, tmp_addr, effective_addr, value
            )
        }
        (Addressing::Branch, AddressingResult::IndInfo(_, base_addr, jmp_addr, _)) => {
            format!("(${:04X}) = {:04X}", base_addr, jmp_addr)
        }
        (_, AddressingResult::IndInfo(_, _, _, _)) => {
            unreachable!();
        }
        //AbsX, AbsY, ZpgX, ZpgY
        (addressing, AddressingResult::AbsZpgInfo(addr, value)) => {
            let reg = match addressing {
                Addressing::AbsX | Addressing::ZpgX => "X",
                Addressing::AbsY | Addressing::ZpgY => "Y",
                _ => unreachable!(),
            };

            match addressing {
                Addressing::AbsX | Addressing::AbsY => format!(
                    "${:02X}{:02X},{} @ {:04X} = {:02X}",
                    arg.args[1], arg.args[0], reg, addr, value
                ),
                Addressing::ZpgX | Addressing::ZpgY => format!(
                    "${:02X},{} @ {:02X} = {:02X}",
                    arg.args[0], reg, addr as u8, value
                ),
                _ => unreachable!(),
            }
        }
        (_, AddressingResult::RegA) => "A".to_string(),
        (_, AddressingResult::None) => "".to_string(),
    };

    format!(
        "{:04X}  {:02X} {:<5} {:>4} {}",
        pc, op_hex, arg_hex, op_str, arg_mean
    )
}
