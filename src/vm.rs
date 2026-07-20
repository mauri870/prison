const INSTRUCTION_BUDGET: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Cooperate,
    Defect,
}

#[derive(Debug, Clone, Copy)]
pub enum PseudoReg {
    LastSelf,
    LastOpp,
    Round,
    ScoreSelf,
    ScoreOpp,
    OppId,
    LastPayoffSelf,
}

#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    Nop,
    Halt,
    Mov { rd: u8, rs: u8 },
    Loadi { rd: u8, imm: i8 },
    LoadScratch { rd: u8, rs: u8 },
    StoreScratch { rs: u8, rd: u8 },
    LoadMemory { rd: u8, rs: u8 },
    StoreMemory { rs: u8, rd: u8 },
    LoadPseudo { rd: u8, which: PseudoReg },
    Add { rd: u8, rs: u8 },
    Sub { rd: u8, rs: u8 },
    Inc { rd: u8 },
    And { rd: u8, rs: u8 },
    Or  { rd: u8, rs: u8 },
    Xor { rd: u8, rs: u8 },
    Cmp  { ra: u8, rb: u8 },
    Cmpi { ra: u8, imm: i8 },
    Test { rd: u8, imm: i8 },
    Jeq(u8),
    Jne(u8),
    Jlt(u8),
    Jgt(u8),
    Jmp(u8),
    Rdrand { rd: u8 },
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

pub struct Observation {
    pub last_self: Option<Action>,
    pub last_opp: Option<Action>,
    pub round: u8,
    pub score_self: i8,
    pub score_opp: i8,
    pub opp_id: u8,
    pub last_payoff_self: i8,
}

impl Default for Observation {
    fn default() -> Self {
        Self {
            last_self: None,
            last_opp: None,
            round: 0,
            score_self: 0,
            score_opp: 0,
            opp_id: 0,
            last_payoff_self: 0,
        }
    }
}

pub struct StrategyMemory {
    pub memory: [i8; 256],
}

impl StrategyMemory {
    pub fn new() -> Self {
        Self { memory: [0; 256] }
    }
}

pub struct Vm {
    pub regs: [i8; 4],
    scratch: [i8; 256],
    pc: u8,
    flags: Flags,
    rng: u64,
}

impl Vm {
    pub fn new(seed: u64) -> Self {
        Self {
            regs: [0; 4],
            scratch: [0; 256],
            pc: 0,
            flags: Flags::default(),
            rng: if seed == 0 { 1 } else { seed },
        }
    }

    pub fn reset_for_match(&mut self) {
        self.regs = [0; 4];
        self.scratch = [0; 256];
        self.pc = 0;
        self.flags = Flags::default();
    }

    pub fn run_round(
        &mut self,
        program: &[Opcode],
        obs: &Observation,
        mem: &mut StrategyMemory,
    ) -> Result<Action, VmError> {
        self.pc = 0;
        for _ in 0..INSTRUCTION_BUDGET {
            match program.get(self.pc as usize) {
                None => return Err(VmError::PcOutOfBounds),
                Some(&op) => {
                    if let Some(action) = self.execute(op, obs, mem) {
                        return Ok(action);
                    }
                }
            }
        }
        Ok(Action::Cooperate)
    }

    fn execute(&mut self, op: Opcode, obs: &Observation, mem: &mut StrategyMemory) -> Option<Action> {
        match op {
            Opcode::Nop => {
                self.pc = self.pc.wrapping_add(1);
                None
            }
            Opcode::Halt => Some(Action::Cooperate),
            Opcode::Mov { rd, rs } => {
                self.regs[rd as usize] = self.regs[rs as usize];
                self.pc = self.pc.wrapping_add(1);
                None
            }
            Opcode::Loadi { rd, imm } => {
                self.regs[rd as usize] = imm;
                self.pc = self.pc.wrapping_add(1);
                None
            }
            Opcode::LoadScratch { rd, rs } => {
                self.regs[rd as usize] = self.scratch[self.regs[rs as usize] as u8 as usize];
                self.pc = self.pc.wrapping_add(1);
                None
            }
            Opcode::StoreScratch { rs, rd } => {
                self.scratch[self.regs[rs as usize] as u8 as usize] = self.regs[rd as usize];
                self.pc = self.pc.wrapping_add(1);
                None
            }
            Opcode::LoadMemory { rd, rs } => {
                self.regs[rd as usize] = mem.memory[self.regs[rs as usize] as u8 as usize];
                self.pc = self.pc.wrapping_add(1);
                None
            }
            Opcode::StoreMemory { rs, rd } => {
                mem.memory[self.regs[rs as usize] as u8 as usize] = self.regs[rd as usize];
                self.pc = self.pc.wrapping_add(1);
                None
            }
            Opcode::LoadPseudo { rd, which } => {
                self.regs[rd as usize] = load_pseudo(which, obs);
                self.pc = self.pc.wrapping_add(1);
                None
            }
            Opcode::Add { rd, rs } => {
                self.regs[rd as usize] = self.regs[rd as usize].wrapping_add(self.regs[rs as usize]);
                self.pc = self.pc.wrapping_add(1);
                None
            }
            Opcode::Sub { rd, rs } => {
                self.regs[rd as usize] = self.regs[rd as usize].wrapping_sub(self.regs[rs as usize]);
                self.pc = self.pc.wrapping_add(1);
                None
            }
            Opcode::Inc { rd } => {
                self.regs[rd as usize] = self.regs[rd as usize].wrapping_add(1);
                self.pc = self.pc.wrapping_add(1);
                None
            }
            Opcode::And { rd, rs } => {
                self.regs[rd as usize] &= self.regs[rs as usize];
                self.pc = self.pc.wrapping_add(1);
                None
            }
            Opcode::Or { rd, rs } => {
                self.regs[rd as usize] |= self.regs[rs as usize];
                self.pc = self.pc.wrapping_add(1);
                None
            }
            Opcode::Xor { rd, rs } => {
                self.regs[rd as usize] ^= self.regs[rs as usize];
                self.pc = self.pc.wrapping_add(1);
                None
            }
            Opcode::Cmp { ra, rb } => {
                let a = self.regs[ra as usize];
                let b = self.regs[rb as usize];
                self.flags.eq = a == b;
                self.flags.lt = a < b;
                self.flags.gt = a > b;
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
            Opcode::Jne(target) => {
                self.pc = if !self.flags.eq { target } else { self.pc.wrapping_add(1) };
                None
            }
            Opcode::Jlt(target) => {
                self.pc = if self.flags.lt { target } else { self.pc.wrapping_add(1) };
                None
            }
            Opcode::Jgt(target) => {
                self.pc = if self.flags.gt { target } else { self.pc.wrapping_add(1) };
                None
            }
            Opcode::Jmp(target) => {
                self.pc = target;
                None
            }
            Opcode::Rdrand { rd } => {
                self.regs[rd as usize] = xorshift64(&mut self.rng) as i8;
                self.pc = self.pc.wrapping_add(1);
                None
            }
            Opcode::Play(action) => Some(action),
        }
    }
}

fn load_pseudo(which: PseudoReg, obs: &Observation) -> i8 {
    match which {
        PseudoReg::LastSelf       => obs.last_self.map(action_to_i8).unwrap_or(0),
        PseudoReg::LastOpp        => obs.last_opp.map(action_to_i8).unwrap_or(0),
        PseudoReg::Round          => obs.round as i8,
        PseudoReg::ScoreSelf      => obs.score_self,
        PseudoReg::ScoreOpp       => obs.score_opp,
        PseudoReg::OppId          => obs.opp_id as i8,
        PseudoReg::LastPayoffSelf => obs.last_payoff_self,
    }
}

fn action_to_i8(action: Action) -> i8 {
    match action {
        Action::Cooperate => 0,
        Action::Defect    => 1,
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

    fn run(program: Vec<Opcode>) -> Action {
        run_with_obs(program, Observation::default())
    }

    fn run_with_obs(program: Vec<Opcode>, obs: Observation) -> Action {
        let mut vm = Vm::new(1);
        let mut mem = StrategyMemory::new();
        vm.run_round(&program, &obs, &mut mem).unwrap()
    }

    fn run_with_vm(mut vm: Vm, program: Vec<Opcode>) -> Action {
        let mut mem = StrategyMemory::new();
        vm.run_round(&program, &Observation::default(), &mut mem).unwrap()
    }

    #[test]
    fn play_cooperate() {
        assert_eq!(run(vec![Opcode::Play(Action::Cooperate)]), Action::Cooperate);
    }

    #[test]
    fn play_defect() {
        assert_eq!(run(vec![Opcode::Play(Action::Defect)]), Action::Defect);
    }

    #[test]
    fn nop_does_not_terminate() {
        assert_eq!(run(vec![Opcode::Nop, Opcode::Play(Action::Cooperate)]), Action::Cooperate);
    }

    #[test]
    fn halt_defaults_to_cooperate() {
        assert_eq!(run(vec![Opcode::Halt]), Action::Cooperate);
    }

    #[test]
    fn loadi_sets_register() {
        let program = vec![
            Opcode::Loadi { rd: 0, imm: 42 },
            Opcode::Cmpi { ra: 0, imm: 42 },
            Opcode::Jeq(4),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run(program), Action::Cooperate);
    }

    #[test]
    fn mov_copies_register() {
        let mut vm = Vm::new(1);
        vm.regs[1] = 7;
        let program = vec![
            Opcode::Mov { rd: 0, rs: 1 },
            Opcode::Cmpi { ra: 0, imm: 7 },
            Opcode::Jeq(4),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run_with_vm(vm, program), Action::Cooperate);
    }

    #[test]
    fn store_scratch_and_load_scratch() {
        let mut vm = Vm::new(1);
        vm.regs[1] = 5;  // index
        vm.regs[0] = 99; // value to store
        let mut mem = StrategyMemory::new();
        let program = vec![
            Opcode::StoreScratch { rs: 1, rd: 0 }, // scratch[5] = 99
            Opcode::LoadScratch { rd: 2, rs: 1 },  // R2 = scratch[5]
            Opcode::Cmpi { ra: 2, imm: 99 },
            Opcode::Jeq(5),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(vm.run_round(&program, &Observation::default(), &mut mem).unwrap(), Action::Cooperate);
    }

    #[test]
    fn store_memory_and_load_memory() {
        let mut vm = Vm::new(1);
        vm.regs[1] = 3;   // index
        vm.regs[0] = 77;  // value to store
        let mut mem = StrategyMemory::new();
        let program = vec![
            Opcode::StoreMemory { rs: 1, rd: 0 }, // memory[3] = 77
            Opcode::LoadMemory { rd: 2, rs: 1 },  // R2 = memory[3]
            Opcode::Cmpi { ra: 2, imm: 77 },
            Opcode::Jeq(5),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(vm.run_round(&program, &Observation::default(), &mut mem).unwrap(), Action::Cooperate);
    }

    #[test]
    fn load_pseudo_round() {
        let obs = Observation { round: 5, ..Observation::default() };
        let program = vec![
            Opcode::LoadPseudo { rd: 0, which: PseudoReg::Round },
            Opcode::Cmpi { ra: 0, imm: 5 },
            Opcode::Jeq(4),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run_with_obs(program, obs), Action::Cooperate);
    }

    #[test]
    fn load_pseudo_lastopp_none_is_zero() {
        let obs = Observation { last_opp: None, ..Observation::default() };
        let program = vec![
            Opcode::LoadPseudo { rd: 0, which: PseudoReg::LastOpp },
            Opcode::Cmpi { ra: 0, imm: 0 },
            Opcode::Jeq(4),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run_with_obs(program, obs), Action::Cooperate);
    }

    #[test]
    fn load_pseudo_lastopp_defect_is_one() {
        let obs = Observation { last_opp: Some(Action::Defect), ..Observation::default() };
        let program = vec![
            Opcode::LoadPseudo { rd: 0, which: PseudoReg::LastOpp },
            Opcode::Cmpi { ra: 0, imm: 1 },
            Opcode::Jeq(4),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run_with_obs(program, obs), Action::Cooperate);
    }

    #[test]
    fn load_pseudo_lastself() {
        let obs = Observation { last_self: Some(Action::Cooperate), ..Observation::default() };
        let program = vec![
            Opcode::LoadPseudo { rd: 0, which: PseudoReg::LastSelf },
            Opcode::Cmpi { ra: 0, imm: 0 },
            Opcode::Jeq(4),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run_with_obs(program, obs), Action::Cooperate);
    }

    #[test]
    fn load_pseudo_scoreself() {
        let obs = Observation { score_self: 12, ..Observation::default() };
        let program = vec![
            Opcode::LoadPseudo { rd: 0, which: PseudoReg::ScoreSelf },
            Opcode::Cmpi { ra: 0, imm: 12 },
            Opcode::Jeq(4),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run_with_obs(program, obs), Action::Cooperate);
    }

    #[test]
    fn load_pseudo_scoreopp() {
        let obs = Observation { score_opp: 9, ..Observation::default() };
        let program = vec![
            Opcode::LoadPseudo { rd: 0, which: PseudoReg::ScoreOpp },
            Opcode::Cmpi { ra: 0, imm: 9 },
            Opcode::Jeq(4),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run_with_obs(program, obs), Action::Cooperate);
    }

    #[test]
    fn load_pseudo_oppid() {
        let obs = Observation { opp_id: 3, ..Observation::default() };
        let program = vec![
            Opcode::LoadPseudo { rd: 0, which: PseudoReg::OppId },
            Opcode::Cmpi { ra: 0, imm: 3 },
            Opcode::Jeq(4),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run_with_obs(program, obs), Action::Cooperate);
    }

    #[test]
    fn load_pseudo_lastpayoffself() {
        let obs = Observation { last_payoff_self: 5, ..Observation::default() };
        let program = vec![
            Opcode::LoadPseudo { rd: 0, which: PseudoReg::LastPayoffSelf },
            Opcode::Cmpi { ra: 0, imm: 5 },
            Opcode::Jeq(4),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run_with_obs(program, obs), Action::Cooperate);
    }

    #[test]
    fn add() {
        let mut vm = Vm::new(1);
        vm.regs[0] = 3;
        vm.regs[1] = 4;
        let program = vec![
            Opcode::Add { rd: 0, rs: 1 },
            Opcode::Cmpi { ra: 0, imm: 7 },
            Opcode::Jeq(4),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run_with_vm(vm, program), Action::Cooperate);
    }

    #[test]
    fn sub() {
        let mut vm = Vm::new(1);
        vm.regs[0] = 10;
        vm.regs[1] = 3;
        let program = vec![
            Opcode::Sub { rd: 0, rs: 1 },
            Opcode::Cmpi { ra: 0, imm: 7 },
            Opcode::Jeq(4),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run_with_vm(vm, program), Action::Cooperate);
    }

    #[test]
    fn inc() {
        let mut vm = Vm::new(1);
        vm.regs[0] = 41;
        let program = vec![
            Opcode::Inc { rd: 0 },
            Opcode::Cmpi { ra: 0, imm: 42 },
            Opcode::Jeq(4),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run_with_vm(vm, program), Action::Cooperate);
    }

    #[test]
    fn and() {
        let mut vm = Vm::new(1);
        vm.regs[0] = 0b1100;
        vm.regs[1] = 0b1010;
        let program = vec![
            Opcode::And { rd: 0, rs: 1 },
            Opcode::Cmpi { ra: 0, imm: 0b1000 },
            Opcode::Jeq(4),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run_with_vm(vm, program), Action::Cooperate);
    }

    #[test]
    fn or() {
        let mut vm = Vm::new(1);
        vm.regs[0] = 0b1100;
        vm.regs[1] = 0b0011;
        let program = vec![
            Opcode::Or { rd: 0, rs: 1 },
            Opcode::Cmpi { ra: 0, imm: 0b1111 },
            Opcode::Jeq(4),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run_with_vm(vm, program), Action::Cooperate);
    }

    #[test]
    fn xor() {
        let mut vm = Vm::new(1);
        vm.regs[0] = 0b1111;
        vm.regs[1] = 0b1010;
        let program = vec![
            Opcode::Xor { rd: 0, rs: 1 },
            Opcode::Cmpi { ra: 0, imm: 0b0101 },
            Opcode::Jeq(4),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run_with_vm(vm, program), Action::Cooperate);
    }

    #[test]
    fn cmp_sets_eq() {
        let mut vm = Vm::new(1);
        vm.regs[0] = 5;
        vm.regs[1] = 5;
        let program = vec![
            Opcode::Cmp { ra: 0, rb: 1 },
            Opcode::Jeq(3),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run_with_vm(vm, program), Action::Cooperate);
    }

    #[test]
    fn cmp_sets_lt() {
        let mut vm = Vm::new(1);
        vm.regs[0] = 3;
        vm.regs[1] = 5;
        let program = vec![
            Opcode::Cmp { ra: 0, rb: 1 },
            Opcode::Jlt(3),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run_with_vm(vm, program), Action::Cooperate);
    }

    #[test]
    fn cmp_sets_gt() {
        let mut vm = Vm::new(1);
        vm.regs[0] = 7;
        vm.regs[1] = 5;
        let program = vec![
            Opcode::Cmp { ra: 0, rb: 1 },
            Opcode::Jgt(3),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run_with_vm(vm, program), Action::Cooperate);
    }

    #[test]
    fn cmpi_sets_flags() {
        let mut vm = Vm::new(1);
        vm.regs[0] = 5;
        let program = vec![
            Opcode::Cmpi { ra: 0, imm: 5 },
            Opcode::Jeq(3),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run_with_vm(vm, program), Action::Cooperate);
    }

    #[test]
    fn test_eq_when_masked_zero() {
        let mut vm = Vm::new(1);
        vm.regs[0] = 2;
        let program = vec![
            Opcode::Test { rd: 0, imm: 1 },
            Opcode::Jeq(3),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run_with_vm(vm, program), Action::Cooperate);
    }

    #[test]
    fn test_not_eq_when_masked_nonzero() {
        let mut vm = Vm::new(1);
        vm.regs[0] = 3;
        let program = vec![
            Opcode::Test { rd: 0, imm: 1 },
            Opcode::Jeq(3),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run_with_vm(vm, program), Action::Defect);
    }

    #[test]
    fn jeq_taken_when_eq() {
        let program = vec![
            Opcode::Cmpi { ra: 0, imm: 0 },
            Opcode::Jeq(3),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run(program), Action::Cooperate);
    }

    #[test]
    fn jeq_not_taken_when_not_eq() {
        let program = vec![
            Opcode::Cmpi { ra: 0, imm: 1 },
            Opcode::Jeq(3),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run(program), Action::Defect);
    }

    #[test]
    fn jne_taken_when_not_eq() {
        let program = vec![
            Opcode::Cmpi { ra: 0, imm: 1 },
            Opcode::Jne(3),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run(program), Action::Cooperate);
    }

    #[test]
    fn jne_not_taken_when_eq() {
        let program = vec![
            Opcode::Cmpi { ra: 0, imm: 0 },
            Opcode::Jne(3),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run(program), Action::Defect);
    }

    #[test]
    fn jlt_taken_when_lt() {
        let mut vm = Vm::new(1);
        vm.regs[0] = -1;
        let program = vec![
            Opcode::Cmpi { ra: 0, imm: 0 },
            Opcode::Jlt(3),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run_with_vm(vm, program), Action::Cooperate);
    }

    #[test]
    fn jlt_not_taken_when_not_lt() {
        let program = vec![
            Opcode::Cmpi { ra: 0, imm: 0 },
            Opcode::Jlt(3),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run(program), Action::Defect);
    }

    #[test]
    fn jgt_taken_when_gt() {
        let mut vm = Vm::new(1);
        vm.regs[0] = 1;
        let program = vec![
            Opcode::Cmpi { ra: 0, imm: 0 },
            Opcode::Jgt(3),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run_with_vm(vm, program), Action::Cooperate);
    }

    #[test]
    fn jgt_not_taken_when_not_gt() {
        let program = vec![
            Opcode::Cmpi { ra: 0, imm: 0 },
            Opcode::Jgt(3),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run(program), Action::Defect);
    }

    #[test]
    fn jmp_unconditional() {
        let program = vec![
            Opcode::Jmp(2),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        assert_eq!(run(program), Action::Cooperate);
    }

    #[test]
    fn rdrand_produces_both_outcomes() {
        let program = vec![
            Opcode::Rdrand { rd: 0 },
            Opcode::Test { rd: 0, imm: 1 },
            Opcode::Jeq(4),
            Opcode::Play(Action::Defect),
            Opcode::Play(Action::Cooperate),
        ];
        let outcomes: Vec<Action> = (1u64..=64)
            .map(|seed| {
                let mut vm = Vm::new(seed);
                let mut mem = StrategyMemory::new();
                vm.run_round(&program, &Observation::default(), &mut mem).unwrap()
            })
            .collect();
        assert!(outcomes.contains(&Action::Cooperate));
        assert!(outcomes.contains(&Action::Defect));
    }
}
