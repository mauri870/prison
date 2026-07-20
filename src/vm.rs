const INSTRUCTION_BUDGET: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Cooperate,
    Defect,
}

#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    Play(Action),
}

#[derive(Debug)]
pub enum VmError {
    PcOutOfBounds,
}

pub struct Vm {
    pc: u8,
}

impl Vm {
    pub fn new() -> Self {
        Self { pc: 0 }
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
            Opcode::Play(action) => Some(action),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play_cooperate() {
        let program = vec![Opcode::Play(Action::Cooperate)];
        assert_eq!(Vm::new().run_round(&program).unwrap(), Action::Cooperate);
    }

    #[test]
    fn play_defect() {
        let program = vec![Opcode::Play(Action::Defect)];
        assert_eq!(Vm::new().run_round(&program).unwrap(), Action::Defect);
    }
}
