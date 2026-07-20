use crate::vm::{Action, Opcode};

#[derive(Debug)]
pub enum DecodeError {
    UnknownOpcode(u8),
}

pub fn decode(bytes: &[u8]) -> Result<Vec<Opcode>, DecodeError> {
    let mut instructions = Vec::new();
    let mut cursor = 0usize;

    while cursor < bytes.len() {
        let (instr, width) = decode_one(bytes, cursor)?;
        instructions.push(instr);
        cursor += width;
    }

    Ok(instructions)
}

fn decode_one(bytes: &[u8], at: usize) -> Result<(Opcode, usize), DecodeError> {
    match bytes[at] {
        0x70 => Ok((Opcode::Play(Action::Cooperate), 1)),
        0x71 => Ok((Opcode::Play(Action::Defect), 1)),
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
}
