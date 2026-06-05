use super::cpu::{Cpu, Status};
use super::nes::Nes;

pub trait BinaryOperation {
    fn calc_and_set(&self, nes: &Nes, value: u8);
}

pub struct ORA;
impl BinaryOperation for ORA {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes, value: u8) {
        let res = nes.cpu.a.get() | value;

        let mut p = nes.cpu.p.get();
        p.set(Status::Z, res == 0);
        p.set(Status::N, res & 0x80 != 0);

        nes.cpu.p.set(p);
        nes.cpu.a.set(res);
    }
}

pub struct AND;
impl BinaryOperation for AND {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes, value: u8) {
        let res = nes.cpu.a.get() & value;

        let mut p = nes.cpu.p.get();
        p.set(Status::Z, res == 0);
        p.set(Status::N, res & 0x80 != 0);

        nes.cpu.p.set(p);
        nes.cpu.a.set(res);
    }
}

pub struct EOR;
impl BinaryOperation for EOR {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes, value: u8) {
        let res = nes.cpu.a.get() ^ value;

        let mut p = nes.cpu.p.get();
        p.set(Status::Z, res == 0);
        p.set(Status::N, res & 0x80 != 0);

        nes.cpu.p.set(p);
        nes.cpu.a.set(res);
    }
}

pub struct ADC;
impl BinaryOperation for ADC {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes, value: u8) {
        // let c = (nes.cpu.p.get() & Status::C != Status::empty()) as u16;
        let c = nes.cpu.p.get().contains(Status::C) as u16;

        let a = nes.cpu.a.get();
        let res = (a as u16) + (value as u16) + c;

        let mut p = nes.cpu.p.get();
        p.set(Status::Z, res & 0xFF == 0);
        p.set(Status::C, res & 0x100 != 0);
        p.set(Status::N, res & 0x80 != 0);
        p.set(Status::V, (a ^ res as u8) & (value ^ res as u8) & 0x80 != 0);

        nes.cpu.p.set(p);
        nes.cpu.a.set(res as u8);
    }
}

pub struct SBC;
impl BinaryOperation for SBC {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes, value: u8) {
        // let c = (nes.cpu.p.get() & Status::C != Status::empty()) as i16;
        let c = nes.cpu.p.get().contains(Status::C) as i16;
        let a = nes.cpu.a.get();
        let not_value = !value;
        let res = (a as i16) + (not_value as i16) + c;

        let mut p = nes.cpu.p.get();
        p.set(Status::Z, res & 0xFF == 0);
        p.set(Status::C, res & 0x100 != 0);
        p.set(Status::N, res & 0x80 != 0);
        p.set(
            Status::V,
            (a ^ res as u8) & (not_value ^ res as u8) & 0x80 != 0,
        );

        nes.cpu.p.set(p);
        nes.cpu.a.set(res as u8);
    }
}

pub struct ASL;
impl BinaryOperation for ASL {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes, value: u8) {
        let a = value;

        let mut p = nes.cpu.p.get();
        p.set(Status::C, a & 0x80 != 0);

        let res = a << 1;

        p.set(Status::Z, res == 0);
        p.set(Status::N, res & 0x80 != 0);

        nes.cpu.p.set(p);
        nes.cpu.a.set(res);
    }
}

impl MemCalcOperation for ASL {
    #[inline(always)]
    fn calc(&self, nes: &Nes, value: u8) -> u8 {
        let mut p = nes.cpu.p.get();
        p.set(Status::C, value & 0x80 != 0);

        let res = value << 1;

        p.set(Status::Z, res == 0);
        p.set(Status::N, res & 0x80 != 0);

        nes.cpu.p.set(p);
        res
    }
}

pub struct LSR;
impl BinaryOperation for LSR {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes, value: u8) {
        let a = value;

        let mut p = nes.cpu.p.get();
        p.set(Status::C, a & 0x01 != 0);

        let res = a >> 1;
        p.set(Status::Z, res == 0);
        p.set(Status::N, res & 0x80 != 0);

        nes.cpu.p.set(p);
        nes.cpu.a.set(res);
    }
}

impl MemCalcOperation for LSR {
    #[inline(always)]
    fn calc(&self, nes: &Nes, value: u8) -> u8 {
        let mut p = nes.cpu.p.get();
        p.set(Status::C, value & 0x01 != 0);

        let res = value >> 1;
        p.set(Status::Z, res == 0);
        p.set(Status::N, res & 0x80 != 0);

        nes.cpu.p.set(p);
        res
    }
}

pub struct ROL;
impl BinaryOperation for ROL {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes, value: u8) {
        let mut p = nes.cpu.p.get();
        let lsb = p.contains(Status::C) as u16;

        let a = ((value as u16) << 1) | lsb;

        p.set(Status::C, a & 0x100 != 0);

        let res = a as u8;

        p.set(Status::Z, res == 0);
        p.set(Status::N, res & 0x80 != 0);

        nes.cpu.p.set(p);
        nes.cpu.a.set(res);
    }
}

impl MemCalcOperation for ROL {
    #[inline(always)]
    fn calc(&self, nes: &Nes, value: u8) -> u8 {
        let lsb = nes.cpu.p.get().contains(Status::C) as u8;
        let mut p = nes.cpu.p.get();
        p.set(Status::C, value & 0x80 != 0);

        let res = (value << 1) | lsb;

        p.set(Status::Z, res == 0);
        p.set(Status::N, res & 0x80 != 0);

        nes.cpu.p.set(p);
        res as u8
    }
}

pub struct ROR;
impl BinaryOperation for ROR {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes, value: u8) {
        let s = (nes.cpu.p.get().contains(Status::C) as u16) << 8;
        let a = (value as u16) | s;

        let mut p = nes.cpu.p.get();

        p.set(Status::C, a & 0x001 != 0);

        let res = (a >> 1) as u8;
        p.set(Status::Z, res == 0);
        p.set(Status::N, res & 0x80 != 0);

        nes.cpu.p.set(p);
        nes.cpu.a.set(res);
    }
}

impl MemCalcOperation for ROR {
    #[inline(always)]
    fn calc(&self, nes: &Nes, value: u8) -> u8 {
        let mut p = nes.cpu.p.get();
        let msb = (p.contains(Status::C) as u8) << 7;

        p.set(Status::C, value & 0x01 != 0);

        let res = ((value >> 1) as u8) | msb;
        p.set(Status::Z, res == 0);
        p.set(Status::N, res & 0x80 != 0);

        nes.cpu.p.set(p);
        res
    }
}

pub struct CMP;
impl BinaryOperation for CMP {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes, value: u8) {
        let a = nes.cpu.a.get();
        let res = a as i16 - value as i16;

        let mut p = nes.cpu.p.get();
        p.set(Status::Z, res == 0);
        p.set(Status::C, res >= 0);
        p.set(Status::N, res & 0x80 != 0);
        nes.cpu.p.set(p);
    }
}

pub struct CPX;
impl BinaryOperation for CPX {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes, value: u8) {
        let x = nes.cpu.x.get();

        let res = x as i16 - value as i16;

        let mut p = nes.cpu.p.get();
        p.set(Status::Z, res == 0);
        p.set(Status::C, res >= 0);
        p.set(Status::N, res & 0x80 != 0);
        nes.cpu.p.set(p);
    }
}

pub struct CPY; //Compare M and Y
impl BinaryOperation for CPY {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes, value: u8) {
        let y = nes.cpu.y.get();

        let res = y as i16 - value as i16;

        let mut p = nes.cpu.p.get();
        p.set(Status::Z, res == 0);
        p.set(Status::C, res >= 0);
        p.set(Status::N, res & 0x80 != 0);
        nes.cpu.p.set(p);
    }
}

pub trait ImplOperation {
    fn calc_and_set(&self, nes: &Nes);
}

pub struct PHP;
impl ImplOperation for PHP {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes) {
        let p = nes.cpu.p.get();
        Cpu::push(nes, p.bits() | 0x10);
    }
}

pub struct CLC;
impl ImplOperation for CLC {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes) {
        nes.cpu.p.update(|p| p & !Status::C);
    }
}

pub struct PLP;
impl ImplOperation for PLP {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes) {
        let mut p = Status::from_bits_truncate(Cpu::pop(nes));
        p.set(Status::B, false);
        p.set(Status::U, true);
        nes.cpu.p.set(p);
    }
}

pub struct SEC;
impl ImplOperation for SEC {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes) {
        nes.cpu.p.update(|p| p | Status::C);
    }
}

pub struct PHA;
impl ImplOperation for PHA {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes) {
        let a = nes.cpu.a.get();
        Cpu::push(nes, a);
    }
}

pub struct CLI;
impl ImplOperation for CLI {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes) {
        nes.cpu.p.update(|p| p & !Status::I);
    }
}

pub struct PLA;
impl ImplOperation for PLA {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes) {
        let a = Cpu::pop(nes);

        let mut p = nes.cpu.p.get();

        p.set(Status::Z, a == 0);
        p.set(Status::N, a & 0x80 != 0);

        nes.cpu.p.set(p);
        nes.cpu.a.set(a);
    }
}

pub struct SEI;
impl ImplOperation for SEI {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes) {
        nes.cpu.p.update(|p| p | Status::I);
    }
}

pub struct DEY;
impl ImplOperation for DEY {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes) {
        let y = ((nes.cpu.y.get() as i16) - 1) as u8;

        let mut p = nes.cpu.p.get();

        p.set(Status::Z, y == 0);
        p.set(Status::N, y & 0x80 != 0);

        nes.cpu.p.set(p);
        nes.cpu.y.set(y as u8);
    }
}

pub struct CLV;
impl ImplOperation for CLV {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes) {
        nes.cpu.p.update(|p| p & !Status::V);
    }
}

pub struct INY;
impl ImplOperation for INY {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes) {
        let y = ((nes.cpu.y.get() as u16) + 1) as u8;

        let mut p = nes.cpu.p.get();

        p.set(Status::Z, y == 0);
        p.set(Status::N, y & 0x80 != 0);

        nes.cpu.p.set(p);
        nes.cpu.y.set(y);
    }
}

pub struct CLD;
impl ImplOperation for CLD {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes) {
        nes.cpu.p.update(|p| p & !Status::D);
    }
}

pub struct INX;
impl ImplOperation for INX {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes) {
        let x = ((nes.cpu.x.get() as u16) + 1) as u8;

        let mut p = nes.cpu.p.get();

        p.set(Status::Z, x == 0);
        p.set(Status::N, x & 0x80 != 0);

        nes.cpu.p.set(p);
        nes.cpu.x.set(x);
    }
}

pub struct SED;
impl ImplOperation for SED {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes) {
        nes.cpu.p.update(|p| p | Status::D);
    }
}

pub struct DEX;
impl ImplOperation for DEX {
    #[inline(always)]
    fn calc_and_set(&self, nes: &Nes) {
        let x = ((nes.cpu.x.get() as i16) - 1) as u8;

        let mut p = nes.cpu.p.get();

        p.set(Status::Z, x == 0);
        p.set(Status::N, x & 0x80 != 0);

        nes.cpu.p.set(p);
        nes.cpu.x.set(x);
    }
}

pub struct NOP;
impl ImplOperation for NOP {
    #[inline(always)]
    fn calc_and_set(&self, _: &Nes) {}
}

impl RegOperations for NOP {
    #[inline(always)]
    fn load(&self, _: &Nes, _: u8) {}
    #[inline(always)]
    fn read(&self, _: &Nes) -> u8 {
        0
    }
}

pub trait RegOperations {
    fn load(&self, nes: &Nes, value: u8);
    fn read(&self, nes: &Nes) -> u8;
}

pub struct RegA;
impl RegOperations for RegA {
    #[inline(always)]
    fn load(&self, nes: &Nes, value: u8) {
        let mut p = nes.cpu.p.get();

        p.set(Status::Z, value == 0);
        p.set(Status::N, value & 0x80 != 0);

        nes.cpu.p.set(p);
        nes.cpu.a.set(value);
    }

    #[inline(always)]
    fn read(&self, nes: &Nes) -> u8 {
        nes.cpu.a.get()
    }
}

pub struct RegY;
impl RegOperations for RegY {
    #[inline(always)]
    fn load(&self, nes: &Nes, value: u8) {
        let mut p = nes.cpu.p.get();

        p.set(Status::Z, value == 0);
        p.set(Status::N, value & 0x80 != 0);

        nes.cpu.p.set(p);
        nes.cpu.y.set(value);
    }

    #[inline(always)]
    fn read(&self, nes: &Nes) -> u8 {
        nes.cpu.y.get()
    }
}

pub struct RegX;
impl RegOperations for RegX {
    #[inline(always)]
    fn load(&self, nes: &Nes, value: u8) {
        let mut p = nes.cpu.p.get();

        p.set(Status::Z, value == 0);
        p.set(Status::N, value & 0x80 != 0);

        nes.cpu.p.set(p);
        nes.cpu.x.set(value);
    }

    #[inline(always)]
    fn read(&self, nes: &Nes) -> u8 {
        nes.cpu.x.get()
    }
}

pub struct RegS;
impl RegOperations for RegS {
    #[inline(always)]
    fn load(&self, nes: &Nes, value: u8) {
        nes.cpu.s.set(value);
    }

    #[inline(always)]
    fn read(&self, nes: &Nes) -> u8 {
        nes.cpu.s.get()
    }
}

pub trait MemCalcOperation {
    fn calc(&self, nes: &Nes, value: u8) -> u8;
}

pub struct DEC;
impl MemCalcOperation for DEC {
    #[inline(always)]
    fn calc(&self, nes: &Nes, value: u8) -> u8 {
        let res = (value as i16 - 1) as u8;
        let mut p = nes.cpu.p.get();
        p.set(Status::Z, res == 0);
        p.set(Status::N, res & 0x80 != 0);
        nes.cpu.p.set(p);
        res
    }
}

pub struct INC;
impl MemCalcOperation for INC {
    #[inline(always)]
    fn calc(&self, nes: &Nes, value: u8) -> u8 {
        let res = (value as u16 + 1) as u8;
        let mut p = nes.cpu.p.get();
        p.set(Status::Z, res == 0);
        p.set(Status::N, res & 0x80 != 0);
        nes.cpu.p.set(p);
        res
    }
}

pub trait BranchOperation {
    fn branch(&self, nes: &Nes) -> bool;
}

pub struct BPL;
impl BranchOperation for BPL {
    #[inline(always)]
    fn branch(&self, nes: &Nes) -> bool {
        nes.cpu.p.get() & Status::N != Status::N
    }
}

pub struct BMI;
impl BranchOperation for BMI {
    #[inline(always)]
    fn branch(&self, nes: &Nes) -> bool {
        nes.cpu.p.get() & Status::N == Status::N
    }
}

pub struct BVC;
impl BranchOperation for BVC {
    #[inline(always)]
    fn branch(&self, nes: &Nes) -> bool {
        nes.cpu.p.get() & Status::V != Status::V
    }
}

pub struct BVS;
impl BranchOperation for BVS {
    #[inline(always)]
    fn branch(&self, nes: &Nes) -> bool {
        nes.cpu.p.get() & Status::V == Status::V
    }
}

pub struct BCC;
impl BranchOperation for BCC {
    #[inline(always)]
    fn branch(&self, nes: &Nes) -> bool {
        nes.cpu.p.get() & Status::C != Status::C
    }
}

pub struct BCS;
impl BranchOperation for BCS {
    #[inline(always)]
    fn branch(&self, nes: &Nes) -> bool {
        nes.cpu.p.get() & Status::C == Status::C
    }
}

pub struct BNE;
impl BranchOperation for BNE {
    #[inline(always)]
    fn branch(&self, nes: &Nes) -> bool {
        nes.cpu.p.get() & Status::Z != Status::Z
    }
}

pub struct BEQ;
impl BranchOperation for BEQ {
    #[inline(always)]
    fn branch(&self, nes: &Nes) -> bool {
        nes.cpu.p.get() & Status::Z == Status::Z
    }
}

pub trait TestOperation {
    fn test(&self, nes: &Nes, m: u8);
}

pub struct BIT;
impl TestOperation for BIT {
    fn test(&self, nes: &Nes, m: u8) {
        let res = nes.cpu.a.get() & m;
        let mut p = nes.cpu.p.get();
        p.set(Status::Z, res == 0);
        p.set(Status::N, m & 0b1000_0000 != 0);
        p.set(Status::V, m & 0b0100_0000 != 0);
        nes.cpu.p.set(p);
    }
}

////////////Undocumented Instruction///////////////////

pub struct LAX;
impl RegOperations for LAX {
    #[inline(always)]
    fn load(&self, nes: &Nes, value: u8) {
        let mut p = nes.cpu.p.get();

        p.set(Status::Z, value == 0);
        p.set(Status::N, value & 0x80 != 0);

        nes.cpu.p.set(p);
        nes.cpu.a.set(value);
        nes.cpu.x.set(value);
    }

    #[inline(always)]
    fn read(&self, _: &Nes) -> u8 {
        0
    }
}

pub struct SAX;
impl RegOperations for SAX {
    #[inline(always)]
    fn load(&self, _: &Nes, _: u8) {}

    #[inline(always)]
    fn read(&self, nes: &Nes) -> u8 {
        let a = nes.cpu.a.get();
        let x = nes.cpu.x.get();
        let res = a & x;
        // let mut p = nes.cpu.p.get();
        // p.set(Status::Z, res == 0);
        // p.set(Status::N, res & 0x80 != 0);
        //http://nesdev.com/undocumented_opcodes.txt ??
        // nes.cpu.p.set(p);
        res
    }
}

pub struct DCP;
impl MemCalcOperation for DCP {
    #[inline(always)]
    fn calc(&self, nes: &Nes, value: u8) -> u8 {
        let res = (value as i16 - 1) as u8;
        let a = nes.cpu.a.get();
        let tmp = a as i16 - res as i16;

        let mut p = nes.cpu.p.get();
        p.set(Status::Z, tmp == 0);
        p.set(Status::C, tmp >= 0);
        p.set(Status::N, tmp & 0x80 != 0);
        nes.cpu.p.set(p);
        res
    }
}

pub struct ISB;
impl MemCalcOperation for ISB {
    #[inline(always)]
    fn calc(&self, nes: &Nes, value: u8) -> u8 {
        let tmp = (value as u16 + 1) as u8;
        // let c = (nes.cpu.p.get() & Status::C != Status::empty()) as u16;
        let c = nes.cpu.p.get().contains(Status::C) as u16;
        let a = nes.cpu.a.get();
        let not_value = !tmp;
        let res = (a as u16) + (not_value as u16) + c;

        let mut p = nes.cpu.p.get();
        p.set(Status::Z, res & 0xFF == 0);
        p.set(Status::C, res & 0x100 != 0);
        p.set(Status::N, res & 0x80 != 0);
        p.set(
            Status::V,
            (a ^ res as u8) & (not_value ^ res as u8) & 0x80 != 0,
        );

        nes.cpu.p.set(p);
        nes.cpu.a.set(res as u8);
        tmp
    }
}

pub struct SLO;
impl MemCalcOperation for SLO {
    #[inline(always)]
    fn calc(&self, nes: &Nes, value: u8) -> u8 {
        let mut p = nes.cpu.p.get();
        p.set(Status::C, value & 0x80 != 0);

        let tmp = value << 1;
        let res = nes.cpu.a.get() | tmp;

        p.set(Status::Z, res == 0);
        p.set(Status::N, res & 0x80 != 0);

        nes.cpu.p.set(p);
        nes.cpu.a.set(res);
        tmp
    }
}

pub struct RLA;
impl MemCalcOperation for RLA {
    #[inline(always)]
    fn calc(&self, nes: &Nes, value: u8) -> u8 {
        let mut p = nes.cpu.p.get();
        let lsb = nes.cpu.p.get().contains(Status::C) as u8;
        p.set(Status::C, value & 0x80 != 0);

        let tmp = (value << 1) | lsb;
        let res = nes.cpu.a.get() & tmp;

        p.set(Status::Z, res == 0);
        p.set(Status::N, res & 0x80 != 0);

        nes.cpu.p.set(p);
        nes.cpu.a.set(res);
        tmp
    }
}

pub struct SRE;
impl MemCalcOperation for SRE {
    #[inline(always)]
    fn calc(&self, nes: &Nes, value: u8) -> u8 {
        let mut p = nes.cpu.p.get();
        p.set(Status::C, value & 0x01 != 0);

        let tmp = value >> 1;
        let res = nes.cpu.a.get() ^ tmp;

        p.set(Status::Z, res == 0);
        p.set(Status::N, res & 0x80 != 0);

        nes.cpu.p.set(p);
        nes.cpu.a.set(res);
        tmp
    }
}

pub struct RRA;
impl MemCalcOperation for RRA {
    #[inline(always)]
    fn calc(&self, nes: &Nes, value: u8) -> u8 {
        let mut p = nes.cpu.p.get();
        let msb = (p.contains(Status::C) as u8) << 7;
        let c = value & 0x01;

        let tmp = (value >> 1) | msb;
        let a = nes.cpu.a.get();
        let res = a as u16 + tmp as u16 + c as u16;

        p.set(Status::C, res & 0x100 != 0);
        p.set(Status::Z, res & 0x00FF == 0);
        p.set(Status::N, res & 0x80 != 0);
        p.set(Status::V, (a ^ res as u8) & (tmp ^ res as u8) & 0x80 != 0);

        nes.cpu.p.set(p);
        nes.cpu.a.set(res as u8);
        tmp
    }
}
