use crate::vm::{Action, Opcode, PseudoReg};

#[derive(Debug)]
pub enum DecodeError {
    UnknownOpcode(u8),
    UnknownPseudoReg(u8),
    InvalidJumpTarget(u8),
}

pub fn decode(bytes: &[u8]) -> Result<Vec<Opcode>, DecodeError> {
    let mut instructions: Vec<Opcode> = Vec::new();
    let mut byte_to_instr = [None::<u8>; 256];
    let mut cursor = 0usize;

    while cursor < bytes.len() {
        byte_to_instr[cursor] = Some(instructions.len() as u8);
        let (instr, width) = decode_one(bytes, cursor)?;
        instructions.push(instr);
        cursor += width;
    }

    for instr in &mut instructions {
        match instr {
            Opcode::Jeq(t) | Opcode::Jne(t) | Opcode::Jlt(t) | Opcode::Jgt(t) | Opcode::Jmp(t) => {
                let byte_target = *t;
                *t = byte_to_instr[byte_target as usize]
                    .ok_or(DecodeError::InvalidJumpTarget(byte_target))?;
            }
            _ => {}
        }
    }

    Ok(instructions)
}

fn decode_one(bytes: &[u8], at: usize) -> Result<(Opcode, usize), DecodeError> {
    match bytes[at] {
        0x00 => Ok((Opcode::Nop, 1)),
        0x01 => Ok((Opcode::Halt, 1)),
        0x10 => Ok((Opcode::Mov { rd: bytes[at+1], rs: bytes[at+2] }, 3)),
        0x11 => Ok((Opcode::Loadi { rd: bytes[at+1], imm: bytes[at+2] as i8 }, 3)),
        0x20 => Ok((Opcode::LoadScratch { rd: bytes[at+1], rs: bytes[at+2] }, 3)),
        0x21 => Ok((Opcode::StoreScratch { rs: bytes[at+1], rd: bytes[at+2] }, 3)),
        0x22 => Ok((Opcode::LoadMemory { rd: bytes[at+1], rs: bytes[at+2] }, 3)),
        0x23 => Ok((Opcode::StoreMemory { rs: bytes[at+1], rd: bytes[at+2] }, 3)),
        0x30 => Ok((Opcode::LoadPseudo { rd: bytes[at+1], which: decode_pseudo(bytes[at+2])? }, 3)),
        0x40 => Ok((Opcode::Add { rd: bytes[at+1], rs: bytes[at+2] }, 3)),
        0x41 => Ok((Opcode::Sub { rd: bytes[at+1], rs: bytes[at+2] }, 3)),
        0x42 => Ok((Opcode::Inc { rd: bytes[at+1] }, 2)),
        0x43 => Ok((Opcode::And { rd: bytes[at+1], rs: bytes[at+2] }, 3)),
        0x44 => Ok((Opcode::Or  { rd: bytes[at+1], rs: bytes[at+2] }, 3)),
        0x45 => Ok((Opcode::Xor { rd: bytes[at+1], rs: bytes[at+2] }, 3)),
        0x50 => Ok((Opcode::Cmp  { ra: bytes[at+1], rb: bytes[at+2] }, 3)),
        0x51 => Ok((Opcode::Cmpi { ra: bytes[at+1], imm: bytes[at+2] as i8 }, 3)),
        0x52 => Ok((Opcode::Test { rd: bytes[at+1], imm: bytes[at+2] as i8 }, 3)),
        0x60 => Ok((Opcode::Jeq(bytes[at+1]), 2)),
        0x61 => Ok((Opcode::Jne(bytes[at+1]), 2)),
        0x62 => Ok((Opcode::Jlt(bytes[at+1]), 2)),
        0x63 => Ok((Opcode::Jgt(bytes[at+1]), 2)),
        0x64 => Ok((Opcode::Jmp(bytes[at+1]), 2)),
        0x70 => Ok((Opcode::Play(Action::Cooperate), 1)),
        0x71 => Ok((Opcode::Play(Action::Defect), 1)),
        0x80 => Ok((Opcode::Rdrand { rd: bytes[at+1] }, 2)),
        b    => Err(DecodeError::UnknownOpcode(b)),
    }
}

fn decode_pseudo(selector: u8) -> Result<PseudoReg, DecodeError> {
    match selector {
        0x0 => Ok(PseudoReg::LastSelf),
        0x1 => Ok(PseudoReg::LastOpp),
        0x2 => Ok(PseudoReg::Round),
        0x3 => Ok(PseudoReg::ScoreSelf),
        0x4 => Ok(PseudoReg::ScoreOpp),
        0x5 => Ok(PseudoReg::OppId),
        0x6 => Ok(PseudoReg::LastPayoffSelf),
        s   => Err(DecodeError::UnknownPseudoReg(s)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembler::assemble;

    fn compile(src: &str) -> Vec<Opcode> {
        let bytes = assemble(src).expect("assembly failed");
        decode(&bytes).expect("decode failed")
    }

    #[test]
    fn decode_nop() {
        assert!(matches!(compile("NOP\nPLAY COOPERATE")[0], Opcode::Nop));
    }

    #[test]
    fn decode_halt() {
        assert!(matches!(compile("HALT")[0], Opcode::Halt));
    }

    #[test]
    fn decode_mov() {
        assert!(matches!(compile("MOV R1, R2\nPLAY COOPERATE")[0], Opcode::Mov { rd: 1, rs: 2 }));
    }

    #[test]
    fn decode_loadi() {
        assert!(matches!(compile("LOADI R0, #-5\nPLAY COOPERATE")[0], Opcode::Loadi { rd: 0, imm: -5 }));
    }

    #[test]
    fn decode_load_scratch() {
        assert!(matches!(compile("LOAD R0, SCRATCH[R1]\nPLAY COOPERATE")[0], Opcode::LoadScratch { rd: 0, rs: 1 }));
    }

    #[test]
    fn decode_store_scratch() {
        assert!(matches!(compile("STORE SCRATCH[R1], R0\nPLAY COOPERATE")[0], Opcode::StoreScratch { rs: 1, rd: 0 }));
    }

    #[test]
    fn decode_load_memory() {
        assert!(matches!(compile("LOAD R0, MEMORY[R1]\nPLAY COOPERATE")[0], Opcode::LoadMemory { rd: 0, rs: 1 }));
    }

    #[test]
    fn decode_store_memory() {
        assert!(matches!(compile("STORE MEMORY[R1], R0\nPLAY COOPERATE")[0], Opcode::StoreMemory { rs: 1, rd: 0 }));
    }

    #[test]
    fn decode_load_pseudo_all() {
        let names = ["LASTSELF", "LASTOPP", "ROUND", "SCORESELF", "SCOREOPP", "OPPID", "LASTPAYOFFSELF"];
        let variants = [
            PseudoReg::LastSelf, PseudoReg::LastOpp, PseudoReg::Round,
            PseudoReg::ScoreSelf, PseudoReg::ScoreOpp, PseudoReg::OppId,
            PseudoReg::LastPayoffSelf,
        ];
        for (name, variant) in names.iter().zip(variants.iter()) {
            let src = format!("LOAD R0, {name}\nPLAY COOPERATE");
            let program = compile(&src);
            assert!(matches!(program[0], Opcode::LoadPseudo { rd: 0, which } if std::mem::discriminant(&which) == std::mem::discriminant(variant)));
        }
    }

    #[test]
    fn decode_add() {
        assert!(matches!(compile("ADD R0, R1\nPLAY COOPERATE")[0], Opcode::Add { rd: 0, rs: 1 }));
    }

    #[test]
    fn decode_sub() {
        assert!(matches!(compile("SUB R0, R1\nPLAY COOPERATE")[0], Opcode::Sub { rd: 0, rs: 1 }));
    }

    #[test]
    fn decode_inc() {
        assert!(matches!(compile("INC R2\nPLAY COOPERATE")[0], Opcode::Inc { rd: 2 }));
    }

    #[test]
    fn decode_and() {
        assert!(matches!(compile("AND R0, R1\nPLAY COOPERATE")[0], Opcode::And { rd: 0, rs: 1 }));
    }

    #[test]
    fn decode_or() {
        assert!(matches!(compile("OR R0, R1\nPLAY COOPERATE")[0], Opcode::Or { rd: 0, rs: 1 }));
    }

    #[test]
    fn decode_xor() {
        assert!(matches!(compile("XOR R0, R1\nPLAY COOPERATE")[0], Opcode::Xor { rd: 0, rs: 1 }));
    }

    #[test]
    fn decode_cmp() {
        assert!(matches!(compile("CMP R0, R1\nPLAY COOPERATE")[0], Opcode::Cmp { ra: 0, rb: 1 }));
    }

    #[test]
    fn decode_cmpi() {
        assert!(matches!(compile("CMPI R1, #-3\nPLAY COOPERATE")[0], Opcode::Cmpi { ra: 1, imm: -3 }));
    }

    #[test]
    fn decode_cmp_consts() {
        // CMP Ra, DEFECT resolves to CMPI Ra, #1 via the consts ruledef.
        assert!(matches!(compile("CMP R0, DEFECT\nPLAY COOPERATE")[0], Opcode::Cmpi { ra: 0, imm: 1 }));
        assert!(matches!(compile("CMP R0, COOPERATE\nPLAY COOPERATE")[0], Opcode::Cmpi { ra: 0, imm: 0 }));
    }

    #[test]
    fn decode_test() {
        assert!(matches!(compile("TEST R2, #1\nPLAY COOPERATE")[0], Opcode::Test { rd: 2, imm: 1 }));
    }

    #[test]
    fn decode_jne() {
        let program = compile("JNE label\nlabel:\n    PLAY COOPERATE");
        assert!(matches!(program[0], Opcode::Jne(1)));
    }

    #[test]
    fn decode_jlt() {
        let program = compile("JLT label\nlabel:\n    PLAY COOPERATE");
        assert!(matches!(program[0], Opcode::Jlt(1)));
    }

    #[test]
    fn decode_jgt() {
        let program = compile("JGT label\nlabel:\n    PLAY COOPERATE");
        assert!(matches!(program[0], Opcode::Jgt(1)));
    }

    #[test]
    fn decode_jmp() {
        let program = compile("JMP label\nlabel:\n    PLAY COOPERATE");
        assert!(matches!(program[0], Opcode::Jmp(1)));
    }

    #[test]
    fn decode_jeq_target_rewritten() {
        // Byte layout:
        //   0: RDRAND R0      (2 bytes, instr 0)
        //   2: TEST R0, #1    (3 bytes, instr 1)
        //   5: JEQ label      (2 bytes, instr 2) -> byte 7 = instr 3
        //   7: PLAY COOPERATE (1 byte,  instr 3)
        let program = compile("RDRAND R0\nTEST R0, #1\nJEQ label\nlabel:\n    PLAY COOPERATE");
        assert!(matches!(program[2], Opcode::Jeq(3)));
    }

    #[test]
    fn decode_rdrand() {
        assert!(matches!(compile("RDRAND R0\nPLAY COOPERATE")[0], Opcode::Rdrand { rd: 0 }));
    }

    #[test]
    fn decode_play_cooperate() {
        assert!(matches!(compile("PLAY COOPERATE")[..], [Opcode::Play(Action::Cooperate)]));
    }

    #[test]
    fn decode_play_defect() {
        assert!(matches!(compile("PLAY DEFECT")[..], [Opcode::Play(Action::Defect)]));
    }
}
