use crate::match_runner::{run_match, MatchResult};
use crate::vm::{Opcode, VmError};

pub struct TournamentResult {
    pub standings: Vec<(String, i32)>,
}

pub fn run_tournament(
    strategies: &[(String, Vec<Opcode>)],
    rounds: u32,
    seed: u64,
) -> Result<TournamentResult, VmError> {
    let n = strategies.len();
    let mut totals = vec![0i32; n];

    for i in 0..n {
        for j in (i + 1)..n {
            let match_seed = seed
                .wrapping_add((i * n + j) as u64)
                .wrapping_mul(0x9e3779b97f4a7c15);
            let result: MatchResult = run_match(&strategies[i].1, &strategies[j].1, rounds, match_seed)?;
            totals[i] += result.total_a;
            totals[j] += result.total_b;
        }
    }

    let mut standings: Vec<(String, i32)> = strategies
        .iter()
        .zip(totals.iter())
        .map(|((name, _), &score)| (name.clone(), score))
        .collect();
    standings.sort_by(|a, b| b.1.cmp(&a.1));

    Ok(TournamentResult { standings })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembler::assemble;
    use crate::decoder::decode;

    fn compile(name: &str, src: &str) -> (String, Vec<Opcode>) {
        let bytes = assemble(src).expect("assembly failed");
        (name.to_string(), decode(&bytes).expect("decode failed"))
    }

    #[test]
    fn all_cooperate_equal_scores() {
        let strategies = vec![
            compile("a", include_str!("../strategies/coop.asm")),
            compile("b", include_str!("../strategies/coop.asm")),
            compile("c", include_str!("../strategies/coop.asm")),
        ];
        let result = run_tournament(&strategies, 10, 1).unwrap();
        // Each plays 2 matches against other cooperators: 10*3*2 = 60 per strategy
        let scores: Vec<i32> = result.standings.iter().map(|(_, s)| *s).collect();
        assert!(scores.iter().all(|&s| s == 60), "expected 60 each, got {scores:?}");
    }

    #[test]
    fn defect_beats_cooperate() {
        let strategies = vec![
            compile("coop", include_str!("../strategies/coop.asm")),
            compile("defect", include_str!("../strategies/defect.asm")),
        ];
        let result = run_tournament(&strategies, 1, 1).unwrap();
        let defect_score = result.standings.iter().find(|(n, _)| n == "defect").unwrap().1;
        let coop_score  = result.standings.iter().find(|(n, _)| n == "coop").unwrap().1;
        assert!(defect_score > coop_score);
        assert_eq!(result.standings[0].0, "defect");
    }

    #[test]
    fn single_strategy_no_matches() {
        let strategies = vec![compile("lone", include_str!("../strategies/coop.asm"))];
        let result = run_tournament(&strategies, 10, 1).unwrap();
        assert_eq!(result.standings[0].1, 0);
    }
}
