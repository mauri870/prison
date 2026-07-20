use crate::vm::{Action, Opcode};

#[derive(Debug)]
pub enum DecodeError {
    UnknownOpcode(u8),
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
        if let Opcode::Jeq(target) = instr {
            let byte_target = *target;
            *target = byte_to_instr[byte_target as usize]
                .ok_or(DecodeError::InvalidJumpTarget(byte_target))?;
        }
    }

    Ok(instructions)
}

fn decode_one(bytes: &[u8], at: usize) -> Result<(Opcode, usize), DecodeError> {
    match bytes[at] {
        0x51 => Ok((Opcode::Cmpi { ra: bytes[at+1], imm: bytes[at+2] as i8 }, 3)),
        0x52 => Ok((Opcode::Test { rd: bytes[at+1], imm: bytes[at+2] as i8 }, 3)),
        0x60 => Ok((Opcode::Jeq(bytes[at+1]), 2)),
        0x70 => Ok((Opcode::Play(Action::Cooperate), 1)),
        0x71 => Ok((Opcode::Play(Action::Defect), 1)),
        0x80 => Ok((Opcode::Rdrand { rd: bytes[at+1] }, 2)),
        b    => Err(DecodeError::UnknownOpcode(b)),
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
    fn decode_play_cooperate() {
        let program = compile("PLAY COOPERATE");
        assert!(matches!(program[..], [Opcode::Play(Action::Cooperate)]));
    }

    #[test]
    fn decode_play_defect() {
        let program = compile("PLAY DEFECT");
        assert!(matches!(program[..], [Opcode::Play(Action::Defect)]));
    }

    #[test]
    fn decode_rdrand() {
        let program = compile("RDRAND R0\nPLAY COOPERATE");
        assert!(matches!(program[0], Opcode::Rdrand { rd: 0 }));
    }

    #[test]
    fn decode_cmpi() {
        let program = compile("CMPI R1, #-3\nPLAY COOPERATE");
        assert!(matches!(program[0], Opcode::Cmpi { ra: 1, imm: -3 }));
    }

    #[test]
    fn decode_test() {
        let program = compile("TEST R2, #1\nPLAY COOPERATE");
        assert!(matches!(program[0], Opcode::Test { rd: 2, imm: 1 }));
    }

    #[test]
    fn decode_jeq_target_rewritten() {
        // Byte layout:
        //   0: RDRAND R0      (2 bytes, instr 0)
        //   2: TEST R0, #1    (3 bytes, instr 1)
        //   5: JEQ label      (2 bytes, instr 2) -> byte 7 = instr 3
        //   7: PLAY COOPERATE (1 byte,  instr 3)
        let program = compile(
            "RDRAND R0\nTEST R0, #1\nJEQ label\nlabel:\n    PLAY COOPERATE",
        );
        assert!(matches!(program[2], Opcode::Jeq(3)));
    }
}
