use crate::vm::{Action, Observation, Opcode, StrategyMemory, Vm, VmError};

pub struct RoundResult {
    pub action_a: Action,
    pub action_b: Action,
    pub score_a: i8,
    pub score_b: i8,
}

pub struct MatchResult {
    pub rounds: Vec<RoundResult>,
    pub total_a: i32,
    pub total_b: i32,
}

pub fn run_match(
    program_a: &[Opcode],
    program_b: &[Opcode],
    rounds: u32,
    seed: u64,
) -> Result<MatchResult, VmError> {
    let mut vm_a = Vm::new(seed);
    let mut vm_b = Vm::new(seed);
    let mut mem_a = StrategyMemory::new();
    let mut mem_b = StrategyMemory::new();
    vm_a.reset_for_match();
    vm_b.reset_for_match();

    let mut obs_a = Observation::default();
    let mut obs_b = Observation::default();
    let mut total_a = 0i32;
    let mut total_b = 0i32;
    let mut round_results = Vec::with_capacity(rounds as usize);

    for round in 0..rounds {
        obs_a.round = round as u8;
        obs_b.round = round as u8;
        obs_a.opp_id = 1;
        obs_b.opp_id = 0;

        let action_a = vm_a.run_round(program_a, &obs_a, &mut mem_a)?;
        let action_b = vm_b.run_round(program_b, &obs_b, &mut mem_b)?;

        let (score_a, score_b) = payoff(action_a, action_b);
        total_a += score_a as i32;
        total_b += score_b as i32;

        obs_a.last_self = Some(action_a);
        obs_a.last_opp = Some(action_b);
        obs_a.score_self = total_a.min(i8::MAX as i32) as i8;
        obs_a.score_opp = total_b.min(i8::MAX as i32) as i8;
        obs_a.last_payoff_self = score_a;

        obs_b.last_self = Some(action_b);
        obs_b.last_opp = Some(action_a);
        obs_b.score_self = total_b.min(i8::MAX as i32) as i8;
        obs_b.score_opp = total_a.min(i8::MAX as i32) as i8;
        obs_b.last_payoff_self = score_b;

        round_results.push(RoundResult { action_a, action_b, score_a, score_b });
    }

    Ok(MatchResult { rounds: round_results, total_a, total_b })
}

pub fn payoff(a: Action, b: Action) -> (i8, i8) {
    match (a, b) {
        (Action::Cooperate, Action::Cooperate) => (3, 3),
        (Action::Cooperate, Action::Defect)    => (0, 5),
        (Action::Defect,    Action::Cooperate) => (5, 0),
        (Action::Defect,    Action::Defect)    => (1, 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembler::assemble;
    use crate::decoder::decode;

    fn compile(src: &str) -> Vec<Opcode> {
        let bytes = assemble(src).expect("assembly failed");
        decode(&bytes).expect("decode failed")
    }

    #[test]
    fn payoff_matrix() {
        assert_eq!(payoff(Action::Cooperate, Action::Cooperate), (3, 3));
        assert_eq!(payoff(Action::Cooperate, Action::Defect),    (0, 5));
        assert_eq!(payoff(Action::Defect,    Action::Cooperate), (5, 0));
        assert_eq!(payoff(Action::Defect,    Action::Defect),    (1, 1));
    }

    #[test]
    fn coop_vs_defect_single_round() {
        let coop   = compile(include_str!("../strategies/coop.asm"));
        let defect = compile(include_str!("../strategies/defect.asm"));
        let result = run_match(&coop, &defect, 1, 1).unwrap();
        assert_eq!(result.total_a, 0);
        assert_eq!(result.total_b, 5);
    }

    #[test]
    fn coop_vs_coop_accumulates_score() {
        let coop = compile(include_str!("../strategies/coop.asm"));
        let result = run_match(&coop, &coop, 10, 1).unwrap();
        assert_eq!(result.total_a, 30);
        assert_eq!(result.total_b, 30);
    }

    #[test]
    fn defect_vs_defect_accumulates_score() {
        let defect = compile(include_str!("../strategies/defect.asm"));
        let result = run_match(&defect, &defect, 10, 1).unwrap();
        assert_eq!(result.total_a, 10);
        assert_eq!(result.total_b, 10);
    }

    #[test]
    fn observation_round_increments() {
        let coop = compile(include_str!("../strategies/coop.asm"));
        let result = run_match(&coop, &coop, 5, 1).unwrap();
        assert_eq!(result.rounds.len(), 5);
    }
}
