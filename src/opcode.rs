#![allow(non_snake_case)]
use num_derive::FromPrimitive;

#[derive(Debug, FromPrimitive, PartialEq, Clone, Copy)]
pub enum Opcode {
    /////////////////////
    ///
    /////////////////////
    BrkImpl = 0x00,
    BplRel = 0x10,
    JsrAbs = 0x20,
    BmiRel = 0x30,
    RtiImpl = 0x40,
    BvcRel = 0x50,
    RtsImpl = 0x60,
    BvsRel = 0x70,
    // * 0x80
    BccRel = 0x90,
    LdyImm = 0xA0,
    BcsRel = 0xB0,
    CpyImm = 0xC0,
    BneRel = 0xD0,
    CpxImm = 0xE0,
    BeqRel = 0xF0,
    /////////////////////
    ///
    /////////////////////
    OraXind = 0x01,
    AndXind = 0x21,
    EorXind = 0x41,
    AdcXind = 0x61,
    StaXind = 0x81,
    LdaXind = 0xA1,
    CmpXind = 0xC1,
    SbcXind = 0xE1,

    OraIndY = 0x11,
    AndIndY = 0x31,
    EorIndY = 0x51,
    AdcIndY = 0x71,
    StaIndY = 0x91,
    LdaIndY = 0xB1,
    CmpIndY = 0xD1,
    SbcIndY = 0xF1,
    /////////////////////
    ///
    /////////////////////
    // 0x02,
    // 0x12,
    // 0x22,
    // 0x32,
    // 0x42,
    // 0x52,
    // 0x62,
    // 0x72,
    // 0x82,
    // 0x92,
    LdxImm = 0xA2,
    // 0xB2,
    // 0xC2,
    // 0xD2,
    // 0xE2,
    // 0xF2,
    /////////////////////
    ///
    /////////////////////
    // 0x04,
    // 0x14,
    BitZpg = 0x24,
    // 0x34,
    // 0x44,
    // 0x54,
    // 0x64,
    // 0x74,
    StyZpg = 0x84,
    StyZpgX = 0x94,
    LdyZpg = 0xA4,
    LdyZpgX = 0xB4,
    CpyZpg = 0xC4,
    // 0xD4,
    CpxZpg = 0xE4,
    // 0xF4
    /////////////////////
    ///
    /////////////////////
    OraZpg = 0x05,
    AndZpg = 0x25,
    EorZpg = 0x45,
    AdcZpg = 0x65,
    StaZpg = 0x85,
    LdaZpg = 0xA5,
    CmpZpg = 0xC5,
    SbcZpg = 0xE5,

    OraZpgX = 0x15,
    AndZpgX = 0x35,
    EorZpgX = 0x55,
    AdcZpgX = 0x75,
    StaZpgX = 0x95,
    LdaZpgX = 0xB5,
    CmpZpgX = 0xD5,
    SbcZpgX = 0xF5,
    /////////////////////
    ///
    /////////////////////
    AslZpg = 0x06,
    RolZpg = 0x26,
    LsrZpg = 0x46,
    RorZpg = 0x66,
    StxZpg = 0x86,
    LdxZpg = 0xA6,
    DecZpg = 0xC6,
    IncZpg = 0xE6,

    AslZpgX = 0x16,
    RolZpgX = 0x36,
    LsrZpgX = 0x56,
    RorZpgX = 0x76,
    StxZpgY = 0x96, //ZpgY !
    LdxZpgY = 0xB6, //ZpgY !
    DecZpgX = 0xD6,
    IncZpgX = 0xF6,
    /////////////////////
    ///
    /////////////////////
    PhpImpl = 0x08,
    ClcImpl = 0x18,
    PlpImpl = 0x28,
    SecImpl = 0x38,
    PhaImpl = 0x48,
    CliImpl = 0x58,
    PlaImpl = 0x68,
    SeiImpl = 0x78,
    DeyImpl = 0x88,
    TyaImpl = 0x98,
    TayImpl = 0xA8,
    ClvImpl = 0xB8,
    InyImpl = 0xC8,
    CldImpl = 0xD8,
    InxImpl = 0xE8,
    SedImpl = 0xF8,
    /////////////////////
    ///
    /////////////////////
    OraImm = 0x09,
    AndImm = 0x29,
    EorImm = 0x49,
    AdcImm = 0x69,
    // StaImm = 0x89,
    LdaImm = 0xA9,
    CmpImm = 0xC9,
    SbcImm = 0xE9,

    OraAbsY = 0x19,
    AndAbsY = 0x39,
    EorAbsY = 0x59,
    AdcAbsY = 0x79,
    StaAbsY = 0x99,
    LdaAbsY = 0xB9,
    CmpAbsY = 0xD9,
    SbcAbsY = 0xF9,
    /////////////////////
    ///
    /////////////////////
    AslA = 0x0A,
    // 0x1A
    RolA = 0x2A,
    // 0x3A
    LsrA = 0x4A,
    // 0x5A
    RorA = 0x6A,
    // 0x7A
    TxaImpl = 0x8A,
    TxsImpl = 0x9A,
    TaxImpl = 0xAA,
    TsxImpl = 0xBA,
    DexImpl = 0xCA,
    // 0xDA
    NopImpl = 0xEA,
    // 0xFA
    /////////////////////
    ///
    /////////////////////
    // 0x0C,
    // 0x1C,
    BitAbs = 0x2C,
    // 0x3C,
    JmpAbs = 0x4C,
    // 0x5C,
    JmpInd = 0x6C,
    // 0x7C
    StyAbs = 0x8C,
    // 0x9C,
    LdyAbs = 0xAC,
    LdyAbsX = 0xBC,
    CpyAbs = 0xCC,
    // 0xDC
    CpxAbs = 0xEC,
    // 0xFC
    /////////////////////
    ///
    /////////////////////
    OraAbs = 0x0D,
    AndAbs = 0x2D,
    EorAbs = 0x4D,
    AdcAbs = 0x6D,
    StaAbs = 0x8D,
    LdaAbs = 0xAD,
    CmpAbs = 0xCD,
    SbcAbs = 0xED,

    OraAbsX = 0x1D,
    AndAbsX = 0x3D,
    EorAbsX = 0x5D,
    AdcAbsX = 0x7D,
    StaAbsX = 0x9D,
    LdaAbsX = 0xBD,
    CmpAbsX = 0xDD,
    SbcAbsX = 0xFD,
    /////////////////////
    ///
    /////////////////////
    AslAbs = 0x0E,
    RolAbs = 0x2E,
    LsrAbs = 0x4E,
    RorAbs = 0x6E,
    StxAbs = 0x8E,
    LdxAbs = 0xAE,
    DecAbs = 0xCE,
    IncAbs = 0xEE,

    AslAbsX = 0x1E,
    RolAbsX = 0x3E,
    LsrAbsX = 0x5E,
    RorAbsX = 0x7E,
    // StxAbsX = 0x9E,
    LdxAbsY = 0xBE, //AbsY !
    DecAbsX = 0xDE,
    IncAbsX = 0xFE,

    ////////Undocumented Instructions////////
    UNopZpg1 = 0x04,
    UNopZpg2 = 0x44,
    UNopZpg3 = 0x64,
    UNopZpgX1 = 0x14,
    UNopZpgX2 = 0x34,
    UNopZpgX3 = 0x54,
    UNopZpgX4 = 0x74,
    UNopZpgX5 = 0xD4,
    UNopZpgX6 = 0xF4,
    UNopImm1 = 0x80,
    UNopImm2 = 0x82,
    UNopImm3 = 0x89,
    UNopImm4 = 0xC2,
    UNopImm5 = 0xE2,

    UNopAbs = 0x0C,
    UNopAbsX1 = 0x1C,
    UNopAbsX2 = 0x3C,
    UNopAbsX3 = 0x5C,
    UNopAbsX4 = 0x7C,
    UNopAbsX5 = 0xDC,
    UNopAbsX6 = 0xFC,

    UNopImpl1 = 0x1A,
    UNopImpl2 = 0x3A,
    UNopImpl3 = 0x5A,
    UNopImpl4 = 0x7A,
    UNopImpl5 = 0xDA,
    UNopImpl6 = 0xFA,

    ULaxZpg = 0xA7,
    ULaxZpgY = 0xB7,
    ULaxAbs = 0xAF,
    ULaxAbsY = 0xBF,
    ULaxXind = 0xA3,
    ULaxIndY = 0xB3,

    USaxZpg = 0x87,
    USaxZpgY = 0x97,
    USaxXind = 0x83,
    USaxAbs = 0x8F,

    USbcImm = 0xEB,

    UDcpZpg = 0xC7,
    UDcpZpgX = 0xD7,
    UDcpAbs = 0xCF,
    UDcpAbsX = 0xDF,
    UDcpAbsY = 0xDB,
    UDcpXind = 0xC3,
    UDcpIndY = 0xD3,

    UIsbZpg = 0xE7,
    UIsbZpgX = 0xF7,
    UIsbAbs = 0xEF,
    UIsbAbsX = 0xFF,
    UIsbAbsY = 0xFB,
    UIsbXind = 0xE3,
    UIsbIndY = 0xF3,

    USloZpg = 0x07,
    USloZpgX = 0x17,
    USloAbs = 0x0F,
    USloAbsX = 0x1F,
    USloAbsY = 0x1B,
    USloXind = 0x03,
    USloIndY = 0x13,

    URlaZpg = 0x27,
    URlaZpgX = 0x37,
    URlaAbs = 0x2F,
    URlaAbsX = 0x3F,
    URlaAbsY = 0x3B,
    URlaXind = 0x23,
    URlaIndY = 0x33,

    USreZpg = 0x47,
    USreZpgX = 0x57,
    USreAbs = 0x4F,
    USreAbsX = 0x5F,
    USreAbsY = 0x5B,
    USreXind = 0x43,
    USreIndY = 0x53,

    URraZpg = 0x67,
    URraZpgX = 0x77,
    URraAbs = 0x6F,
    URraAbsX = 0x7F,
    URraAbsY = 0x7B,
    URraXind = 0x63,
    URraIndY = 0x73,
    // Unknown
}

pub fn OpcodeStr<'a>(op: &Opcode) -> &'a str {
    match op {
        Opcode::BrkImpl => "BRK",
        Opcode::BplRel => "BPL",
        Opcode::JsrAbs => "JSR",
        Opcode::BmiRel => "BMI",
        Opcode::RtiImpl => "RTI",
        Opcode::BvcRel => "BVC",
        Opcode::RtsImpl => "RTS",
        Opcode::BvsRel => "BVS",
        // * 0x80=>"// * 0X80",
        Opcode::BccRel => "BCC",
        Opcode::LdyImm => "LDY",
        Opcode::BcsRel => "BCS",
        Opcode::CpyImm => "CPY",
        Opcode::BneRel => "BNE",
        Opcode::CpxImm => "CPX",
        Opcode::BeqRel => "BEQ",
        Opcode::OraXind => "ORA",
        Opcode::AndXind => "AND",
        Opcode::EorXind => "EOR",
        Opcode::AdcXind => "ADC",
        Opcode::StaXind => "STA",
        Opcode::LdaXind => "LDA",
        Opcode::CmpXind => "CMP",
        Opcode::SbcXind => "SBC",
        Opcode::OraIndY => "ORA",
        Opcode::AndIndY => "AND",
        Opcode::EorIndY => "EOR",
        Opcode::AdcIndY => "ADC",
        Opcode::StaIndY => "STA",
        Opcode::LdaIndY => "LDA",
        Opcode::CmpIndY => "CMP",
        Opcode::SbcIndY => "SBC",
        // 0x02,=>"// 0X02",
        // 0x12,=>"// 0X12",
        // 0x22,=>"// 0X22",
        // 0x32,=>"// 0X32",
        // 0x42,=>"// 0X42",
        // 0x52,=>"// 0X52",
        // 0x62,=>"// 0X62",
        // 0x72,=>"// 0X72",
        // 0x82,=>"// 0X82",
        // 0x92,=>"// 0X92",
        Opcode::LdxImm => "LDX",
        // 0xB2,=>"// 0XB2",
        // 0xC2,=>"// 0XC2",
        // 0xD2,=>"// 0XD2",
        // 0xE2,=>"// 0XE2",
        // 0xF2,=>"// 0XF2",
        // 0x04,=>"// 0X04",
        // 0x14,=>"// 0X14",
        Opcode::BitZpg => "BIT",
        // 0x34,=>"// 0X34",
        // 0x44,=>"// 0X44",
        // 0x54,=>"// 0X54",
        // 0x64,=>"// 0X64",
        // 0x74,=>"// 0X74",
        Opcode::StyZpg => "STY",
        Opcode::StyZpgX => "STY",
        Opcode::LdyZpg => "LDY",
        Opcode::LdyZpgX => "LDY",
        Opcode::CpyZpg => "CPY",
        // 0xD4,=>"// 0XD4",
        Opcode::CpxZpg => "CPX",
        // 0xF4=>"// 0XF4",
        Opcode::OraZpg => "ORA",
        Opcode::AndZpg => "AND",
        Opcode::EorZpg => "EOR",
        Opcode::AdcZpg => "ADC",
        Opcode::StaZpg => "STA",
        Opcode::LdaZpg => "LDA",
        Opcode::CmpZpg => "CMP",
        Opcode::SbcZpg => "SBC",
        Opcode::OraZpgX => "ORA",
        Opcode::AndZpgX => "AND",
        Opcode::EorZpgX => "EOR",
        Opcode::AdcZpgX => "ADC",
        Opcode::StaZpgX => "STA",
        Opcode::LdaZpgX => "LDA",
        Opcode::CmpZpgX => "CMP",
        Opcode::SbcZpgX => "SBC",
        Opcode::AslZpg => "ASL",
        Opcode::RolZpg => "ROL",
        Opcode::LsrZpg => "LSR",
        Opcode::RorZpg => "ROR",
        Opcode::StxZpg => "STX",
        Opcode::LdxZpg => "LDX",
        Opcode::DecZpg => "DEC",
        Opcode::IncZpg => "INC",
        Opcode::AslZpgX => "ASL",
        Opcode::RolZpgX => "ROL",
        Opcode::LsrZpgX => "LSR",
        Opcode::RorZpgX => "ROR",
        Opcode::StxZpgY => "STX",
        Opcode::LdxZpgY => "LDX",
        Opcode::DecZpgX => "DEC",
        Opcode::IncZpgX => "INC",
        Opcode::PhpImpl => "PHP",
        Opcode::ClcImpl => "CLC",
        Opcode::PlpImpl => "PLP",
        Opcode::SecImpl => "SEC",
        Opcode::PhaImpl => "PHA",
        Opcode::CliImpl => "CLI",
        Opcode::PlaImpl => "PLA",
        Opcode::SeiImpl => "SEI",
        Opcode::DeyImpl => "DEY",
        Opcode::TyaImpl => "TYA",
        Opcode::TayImpl => "TAY",
        Opcode::ClvImpl => "CLV",
        Opcode::InyImpl => "INY",
        Opcode::CldImpl => "CLD",
        Opcode::InxImpl => "INX",
        Opcode::SedImpl => "SED",
        Opcode::OraImm => "ORA",
        Opcode::AndImm => "AND",
        Opcode::EorImm => "EOR",
        Opcode::AdcImm => "ADC",
        // StaImm=>"// STA",
        Opcode::LdaImm => "LDA",
        Opcode::CmpImm => "CMP",
        Opcode::SbcImm => "SBC",
        Opcode::OraAbsY => "ORA",
        Opcode::AndAbsY => "AND",
        Opcode::EorAbsY => "EOR",
        Opcode::AdcAbsY => "ADC",
        Opcode::StaAbsY => "STA",
        Opcode::LdaAbsY => "LDA",
        Opcode::CmpAbsY => "CMP",
        Opcode::SbcAbsY => "SBC",
        Opcode::AslA => "ASL",
        // 0x1A=>"// 0X1A",
        Opcode::RolA => "ROL",
        // 0x3A=>"// 0X3A",
        Opcode::LsrA => "LSR",
        // 0x5A=>"// 0X5A",
        Opcode::RorA => "ROR",
        // 0x7A=>"// 0X7A",
        Opcode::TxaImpl => "TXA",
        Opcode::TxsImpl => "TXS",
        Opcode::TaxImpl => "TAX",
        Opcode::TsxImpl => "TSX",
        Opcode::DexImpl => "DEX",
        // 0xDA=>"// 0XDA",
        Opcode::NopImpl => "NOP",
        // 0xFA=>"// 0XFA",
        // 0x0C,=>"// 0X0C",
        // 0x1C,=>"// 0X1C",
        Opcode::BitAbs => "BIT",
        // 0x3C,=>"// 0X3C",
        Opcode::JmpAbs => "JMP",
        // 0x5C,=>"// 0X5C",
        Opcode::JmpInd => "JMP",
        // 0x7C=>"// 0X7C",
        Opcode::StyAbs => "STY",
        // 0x9C,=>"// 0X9C",
        Opcode::LdyAbs => "LDY",
        Opcode::LdyAbsX => "LDY",
        Opcode::CpyAbs => "CPY",
        // 0xDC=>"// 0XDC",
        Opcode::CpxAbs => "CPX",
        // 0xFC=>"// 0XFC",
        Opcode::OraAbs => "ORA",
        Opcode::AndAbs => "AND",
        Opcode::EorAbs => "EOR",
        Opcode::AdcAbs => "ADC",
        Opcode::StaAbs => "STA",
        Opcode::LdaAbs => "LDA",
        Opcode::CmpAbs => "CMP",
        Opcode::SbcAbs => "SBC",
        Opcode::OraAbsX => "ORA",
        Opcode::AndAbsX => "AND",
        Opcode::EorAbsX => "EOR",
        Opcode::AdcAbsX => "ADC",
        Opcode::StaAbsX => "STA",
        Opcode::LdaAbsX => "LDA",
        Opcode::CmpAbsX => "CMP",
        Opcode::SbcAbsX => "SBC",
        Opcode::AslAbs => "ASL",
        Opcode::RolAbs => "ROL",
        Opcode::LsrAbs => "LSR",
        Opcode::RorAbs => "ROR",
        Opcode::StxAbs => "STX",
        Opcode::LdxAbs => "LDX",
        Opcode::DecAbs => "DEC",
        Opcode::IncAbs => "INC",
        Opcode::AslAbsX => "ASL",
        Opcode::RolAbsX => "ROL",
        Opcode::LsrAbsX => "LSR",
        Opcode::RorAbsX => "ROR",
        // StxAbsX
        Opcode::LdxAbsY => "LDX",
        Opcode::DecAbsX => "DEC",
        Opcode::IncAbsX => "INC",
        //Undocumented Instructions
        Opcode::UNopZpg1 | Opcode::UNopZpg2 | Opcode::UNopZpg3 => "*NOP",
        Opcode::UNopZpgX1
        | Opcode::UNopZpgX2
        | Opcode::UNopZpgX3
        | Opcode::UNopZpgX4
        | Opcode::UNopZpgX5
        | Opcode::UNopZpgX6 => "*NOP",
        Opcode::UNopImm1
        | Opcode::UNopImm2
        | Opcode::UNopImm3
        | Opcode::UNopImm4
        | Opcode::UNopImm5 => "*NOP",
        Opcode::UNopAbs => "*NOP",
        Opcode::UNopAbsX1
        | Opcode::UNopAbsX2
        | Opcode::UNopAbsX3
        | Opcode::UNopAbsX4
        | Opcode::UNopAbsX5
        | Opcode::UNopAbsX6 => "*NOP",
        Opcode::UNopImpl1
        | Opcode::UNopImpl2
        | Opcode::UNopImpl3
        | Opcode::UNopImpl4
        | Opcode::UNopImpl5
        | Opcode::UNopImpl6 => "*NOP",
        Opcode::ULaxZpg
        | Opcode::ULaxZpgY
        | Opcode::ULaxAbs
        | Opcode::ULaxAbsY
        | Opcode::ULaxXind
        | Opcode::ULaxIndY => "*LAX",
        Opcode::USaxZpg | Opcode::USaxZpgY | Opcode::USaxXind | Opcode::USaxAbs => "*SAX",
        Opcode::USbcImm => "*SBC",
        Opcode::UDcpZpg
        | Opcode::UDcpZpgX
        | Opcode::UDcpAbs
        | Opcode::UDcpAbsX
        | Opcode::UDcpAbsY
        | Opcode::UDcpXind
        | Opcode::UDcpIndY => "*DCP",
        Opcode::UIsbZpg
        | Opcode::UIsbZpgX
        | Opcode::UIsbAbs
        | Opcode::UIsbAbsX
        | Opcode::UIsbAbsY
        | Opcode::UIsbXind
        | Opcode::UIsbIndY => "*ISB",
        Opcode::USloZpg
        | Opcode::USloZpgX
        | Opcode::USloAbs
        | Opcode::USloAbsX
        | Opcode::USloAbsY
        | Opcode::USloXind
        | Opcode::USloIndY => "*SLO",
        Opcode::URlaZpg
        | Opcode::URlaZpgX
        | Opcode::URlaAbs
        | Opcode::URlaAbsX
        | Opcode::URlaAbsY
        | Opcode::URlaXind
        | Opcode::URlaIndY => "*RLA",
        Opcode::USreZpg
        | Opcode::USreZpgX
        | Opcode::USreAbs
        | Opcode::USreAbsX
        | Opcode::USreAbsY
        | Opcode::USreXind
        | Opcode::USreIndY => "*SRE",
        Opcode::URraZpg
        | Opcode::URraZpgX
        | Opcode::URraAbs
        | Opcode::URraAbsX
        | Opcode::URraAbsY
        | Opcode::URraXind
        | Opcode::URraIndY => "*RRA",
    }
}
