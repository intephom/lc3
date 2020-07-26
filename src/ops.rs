#[derive(Debug)]
pub enum Op {
    Nop,
    Not {
        dst: u16,
        src: u16,
    },
    AddReg {
        dst: u16,
        src1: u16,
        src2: u16,
    },
    AddImm {
        dst: u16,
        src: u16,
        imm: i16,
    },
    AndReg {
        dst: u16,
        src1: u16,
        src2: u16,
    },
    AndImm {
        dst: u16,
        src: u16,
        imm: i16,
    },
    Load {
        dst: u16,
        offset: i16,
    },
    LoadInd {
        dst: u16,
        offset: i16,
    },
    LoadReg {
        dst: u16,
        base: u16,
        offset: i16,
    },
    LoadEffAddr {
        dst: u16,
        offset: i16,
    },
    Store {
        src: u16,
        offset: i16,
    },
    StoreInd {
        src: u16,
        offset: i16,
    },
    StoreReg {
        src: u16,
        base: u16,
        offset: i16,
    },
    Call {
        offset: i16,
    },
    CallReg {
        src: u16,
    },
    Branch {
        n: bool,
        z: bool,
        p: bool,
        offset: i16,
    },
    Jump {
        base: u16,
    },
    Trap {
        vector: u8,
    },
}

fn select_bool(instr: u16, bit: i16) -> bool {
    return select_u16(instr, bit, bit) != 0;
}

fn select_u16(instr: u16, start: i16, end: i16) -> u16 {
    let width = start - end;
    let mask = (2u16.pow((width + 1) as u32) - 1) << end;
    return ((instr & mask) >> end) as u16;
}

fn select_i16(instr: u16, start: i16, end: i16) -> i16 {
    let width = start - end;
    let mask = (2u16.pow((width + 1) as u32) - 1) << end;
    let result = ((instr & mask) >> end) as u16;

    if result & (1 << width) == 0 {
        return result as i16;
    }

    // negative, so sign-extend
    return (result | (2u16.pow((15 - width + 1) as u32) - 1) << width) as i16;
}

pub fn decode(instr: u16) -> Option<Op> {
    return match select_u16(instr, 15, 12) {
        0b0000 => Some(Op::Branch {
            n: select_bool(instr, 11),
            z: select_bool(instr, 10),
            p: select_bool(instr, 9),
            offset: select_i16(instr, 8, 0),
        }),
        0b0001 => match select_bool(instr, 5) {
            false => Some(Op::AddReg {
                dst: select_u16(instr, 11, 9),
                src1: select_u16(instr, 8, 6),
                src2: select_u16(instr, 2, 0),
            }),
            true => Some(Op::AddImm {
                dst: select_u16(instr, 11, 9),
                src: select_u16(instr, 8, 6),
                imm: select_i16(instr, 4, 0),
            }),
        },
        0b0010 => Some(Op::Load {
            dst: select_u16(instr, 11, 9),
            offset: select_i16(instr, 8, 0),
        }),
        0b0011 => Some(Op::Store {
            src: select_u16(instr, 11, 9),
            offset: select_i16(instr, 8, 0),
        }),
        0b0100 => match select_bool(instr, 11) {
            false => Some(Op::CallReg {
                src: select_u16(instr, 8, 6),
            }),
            true => Some(Op::Call {
                offset: select_i16(instr, 10, 0),
            }),
        },
        0b0101 => match select_bool(instr, 5) {
            false => Some(Op::AndReg {
                dst: select_u16(instr, 11, 9),
                src1: select_u16(instr, 8, 6),
                src2: select_u16(instr, 2, 0),
            }),
            true => Some(Op::AndImm {
                dst: select_u16(instr, 11, 9),
                src: select_u16(instr, 8, 6),
                imm: select_i16(instr, 4, 0),
            }),
        },
        0b0110 => Some(Op::LoadReg {
            dst: select_u16(instr, 11, 9),
            base: select_u16(instr, 8, 6),
            offset: select_i16(instr, 5, 0),
        }),
        0b0111 => Some(Op::StoreReg {
            src: select_u16(instr, 11, 9),
            base: select_u16(instr, 8, 6),
            offset: select_i16(instr, 5, 0),
        }),
        //0b1000 => Some(Op::Nop), // TODO
        0b1001 => Some(Op::Not {
            dst: select_u16(instr, 11, 9),
            src: select_u16(instr, 8, 6),
        }),
        0b1010 => Some(Op::LoadInd {
            dst: select_u16(instr, 11, 9),
            offset: select_i16(instr, 8, 0),
        }),
        0b1011 => Some(Op::StoreInd {
            src: select_u16(instr, 11, 9),
            offset: select_i16(instr, 8, 0),
        }),
        0b1100 => Some(Op::Jump {
            base: select_u16(instr, 8, 6),
        }),
        //0b1101 => Some(Op::Nop), // TODO
        0b1110 => Some(Op::LoadEffAddr {
            dst: select_u16(instr, 11, 9),
            offset: select_i16(instr, 8, 0),
        }),
        0b1111 => Some(Op::Trap {
            vector: select_u16(instr, 7, 0) as u8,
        }),
        unknown => {
            println!("Unknown opcode {:b}", unknown);
            None
        }
    };
}
