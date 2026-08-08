use prison::vm::{Action, Observation, Vm};

fn run_strategy(path: &std::path::Path) -> datatest_stable::Result<()> {
    let src = std::fs::read_to_string(path)?;
    let config = parse_config(&src);
    if config.cases.is_empty() {
        return Ok(());
    }

    let bytes = prison::assembler::assemble(&src)
        .unwrap_or_else(|e| panic!("assemble failed: {e}"));
    let program = prison::decoder::decode(&bytes)
        .unwrap_or_else(|e| panic!("decode failed: {e:?}"));
    let file = path.display().to_string();

    for (i, case) in config.cases.iter().enumerate() {
        run_case(&program, case, config.seed, &file, i);
    }

    Ok(())
}

struct Case {
    // Each element: (expected strategy action, opponent action this round).
    // Values 0=Cooperate 1=Defect.
    rounds: Vec<(Action, Action)>,
}

struct Config {
    seed: u64,
    cases: Vec<Case>,
}

fn parse_config(src: &str) -> Config {
    let mut seed = 1u64;
    let mut cases = Vec::new();

    for line in src.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("; Seed:") {
            seed = rest.trim().parse().unwrap_or_else(|_| panic!("invalid seed: {rest}"));
        } else if let Some(rest) = line.strip_prefix("; Expect(self, opp) =") {
            let rounds = parse_rounds(rest.trim(), false);
            if !rounds.is_empty() {
                cases.push(Case { rounds });
            }
        } else if let Some(rest) = line.strip_prefix("; Expect(opp, self) =") {
            let rounds = parse_rounds(rest.trim(), true);
            if !rounds.is_empty() {
                cases.push(Case { rounds });
            }
        }
    }

    Config { seed, cases }
}

fn parse_rounds(s: &str, swap: bool) -> Vec<(Action, Action)> {
    let inner = s.trim_start_matches('[').trim_end_matches(']');
    let mut rounds = Vec::new();
    for token in inner.split("),") {
        let token = token.trim().trim_start_matches('(').trim_end_matches(')');
        if token.is_empty() {
            continue;
        }
        let (left, right) = token.split_once(',').expect("expected comma in pair");
        let (a, b) = (parse_action(left), parse_action(right));
        if swap {
            rounds.push((b, a));
        } else {
            rounds.push((a, b));
        }
    }
    rounds
}

fn parse_action(s: &str) -> Action {
    match s.trim() {
        "0" => Action::Cooperate,
        "1" => Action::Defect,
        v => panic!("unknown action '{v}' (use 0 for Cooperate, 1 for Defect)"),
    }
}

fn run_case(
    program: &[prison::vm::Opcode],
    case: &Case,
    seed: u64,
    file: &str,
    case_index: usize,
) {
    let mut vm = Vm::new(seed);
    vm.reset_for_match();

    let mut last_self: Option<Action> = None;
    let mut last_opp: Option<Action> = None;
    let mut score_self = 0i32;
    let mut score_opp = 0i32;
    let mut last_payoff_self: i8 = 0;

    for (round, &(expected, opp_action)) in case.rounds.iter().enumerate() {
        let obs = Observation {
            last_self,
            last_opp,
            round: round as u8,
            score_self: score_self.min(i8::MAX as i32) as i8,
            score_opp: score_opp.min(i8::MAX as i32) as i8,
            opp_id: 1,
            last_payoff_self,
        };

        let actual = vm
            .run_round(program, &obs)
            .unwrap_or_else(|e| panic!("{file}: case {case_index}, round {round}: {e:?}"));

        assert_eq!(
            actual, expected,
            "{file}: case {case_index}, round {round}: expected {expected:?}, got {actual:?}",
        );

        let (pa, pb) = prison::match_runner::payoff(actual, opp_action);
        last_payoff_self = pa;
        score_self += pa as i32;
        score_opp += pb as i32;
        last_self = Some(actual);
        last_opp = Some(opp_action);
    }
}

datatest_stable::harness!(run_strategy, "strategies", r".*\.asm$");
