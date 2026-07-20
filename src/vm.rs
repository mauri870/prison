const INSTRUCTION_BUDGET: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Cooperate,
    Defect,
}

#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    Rdrand { rd: u8 },
    Cmpi { ra: u8, imm: i8 },
    Test { rd: u8, imm: i8 },
    Jeq(u8),
    Play(Action),
}

#[derive(Debug)]
pub enum VmError {
    PcOutOfBounds,
}

#[derive(Default)]
struct Flags {
    eq: bool,
    lt: bool,
    gt: bool,
}

pub struct Vm {
    regs: [i8; 4],
    pc: u8,
    flags: Flags,
    rng: u64,
}

impl Vm {
    pub fn new(seed: u64) -> Self {
        Self {
            regs: [0; 4],
            pc: 0,
            flags: Flags::default(),
            rng: if seed == 0 { 1 } else { seed },
        }
    }

    pub fn run_round(&mut self, program: &[Opcode]) -> Result<Action, VmError> {
        self.pc = 0;
        for _ in 0..INSTRUCTION_BUDGET {
            match program.get(self.pc as usize) {
                None => return Err(VmError::PcOutOfBounds),
                Some(&op) => {
                    if let Some(action) = self.execute(op) {
                        return Ok(action);
                    }
                }
            }
        }
        Ok(Action::Cooperate)
    }

    fn execute(&mut self, op: Opcode) -> Option<Action> {
        match op {
            Opcode::Rdrand { rd } => {
                self.regs[rd as usize] = xorshift64(&mut self.rng) as i8;
                self.pc = self.pc.wrapping_add(1);
                None
            }
            Opcode::Cmpi { ra, imm } => {
                let a = self.regs[ra as usize];
                self.flags.eq = a == imm;
                self.flags.lt = a < imm;
                self.flags.gt = a > imm;
                self.pc = self.pc.wrapping_add(1);
                None
            }
            Opcode::Test { rd, imm } => {
                let result = self.regs[rd as usize] & imm;
                self.flags.eq = result == 0;
                self.flags.lt = result < 0;
                self.flags.gt = result > 0;
                self.pc = self.pc.wrapping_add(1);
                None
            }
            Opcode::Jeq(target) => {
                self.pc = if self.flags.eq { target } else { self.pc.wrapping_add(1) };
                None
            }
            Opcode::Play(action) => Some(action),
        }
    }
}

fn xorshift64(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play_cooperate() {
        let program = vec![Opcode::Play(Action::Cooperate)];
        assert_eq!(Vm::new(1).run_round(&program).unwrap(), Action::Cooperate);
    }

    #[test]
    fn play_defect() {
        let program = vec![Opcode::Play(Action::Defect)];
        assert_eq!(Vm::new(1).run_round(&program).unwrap(), Action::Defect);
    }

    #[test]
    fn cmpi_sets_flags() {
        // CMPI R0, #5 with R0=5 → eq; PLAY to observe we ran past it.
        let program = vec![
            Opcode::Cmpi { ra: 0, imm: 5 },
            Opcode::Jeq(3),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        let mut vm = Vm::new(1);
        vm.regs[0] = 5;
        assert_eq!(vm.run_round(&program).unwrap(), Action::Cooperate);
    }

    #[test]
    fn test_eq_when_masked_zero() {
        // TEST R0, #1 with R0=2 (even) → result 0 → eq flag → JEQ taken.
        let program = vec![
            Opcode::Test { rd: 0, imm: 1 },
            Opcode::Jeq(3),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        let mut vm = Vm::new(1);
        vm.regs[0] = 2;
        assert_eq!(vm.run_round(&program).unwrap(), Action::Cooperate);
    }

    #[test]
    fn test_not_eq_when_masked_nonzero() {
        // TEST R0, #1 with R0=3 (odd) → result 1 → eq flag clear → JEQ not taken.
        let program = vec![
            Opcode::Test { rd: 0, imm: 1 },
            Opcode::Jeq(3),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        let mut vm = Vm::new(1);
        vm.regs[0] = 3;
        assert_eq!(vm.run_round(&program).unwrap(), Action::Defect);
    }

    #[test]
    fn jeq_taken_when_eq() {
        // CMPI R0, #0 with R0=0 sets eq; JEQ should jump over PLAY DEFECT.
        let program = vec![
            Opcode::Cmpi { ra: 0, imm: 0 },
            Opcode::Jeq(3),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(Vm::new(1).run_round(&program).unwrap(), Action::Cooperate);
    }

    #[test]
    fn jeq_not_taken_when_not_eq() {
        // CMPI R0, #1 with R0=0 clears eq; JEQ falls through to PLAY DEFECT.
        let program = vec![
            Opcode::Cmpi { ra: 0, imm: 1 },
            Opcode::Jeq(3),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(Vm::new(1).run_round(&program).unwrap(), Action::Defect);
    }

    #[test]
    fn rdrand_produces_both_outcomes() {
        // TEST R0, #1 checks the low bit — true 50/50 split from a random integer.
        //   0: RDRAND R0
        //   1: TEST R0, #1
        //   2: JEQ 4         (low bit 0 → even → cooperate)
        //   3: PLAY DEFECT
        //   4: PLAY COOPERATE
        let program = vec![
            Opcode::Rdrand { rd: 0 },
            Opcode::Test { rd: 0, imm: 1 },
            Opcode::Jeq(4),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        let outcomes: Vec<Action> = (1u64..=64)
            .map(|seed| Vm::new(seed).run_round(&program).unwrap())
            .collect();
        assert!(outcomes.contains(&Action::Cooperate));
        assert!(outcomes.contains(&Action::Defect));
    }
}
